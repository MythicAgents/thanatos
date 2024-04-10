import asyncio
import json
import os
import pathlib
import tempfile
import time
from shutil import copytree
from mythic_container.PayloadBuilder import (
    PayloadType,
    SupportedOS,
    BuildParameter,
    BuildParameterType,
    BuildResponse,
    BuildStatus,
)


# Class defining information about the Thanatos payload
class Thanatos(PayloadType):
    name = "thanatos"  # Name of the payload
    file_extension = "exe"  # default file extension to use when creating payloads
    author = "@M_alphaaa"  # authors

    # Platforms that thanatos supports
    supported_os = [
        SupportedOS.Windows,
        SupportedOS.Linux,
    ]
    wrapper = False
    wrapped_payloads = []
    # Description of the payload in Mythic
    note = "Linux and Windows agent written in Rust"

    # Payload does not support dynamic loading
    supports_dynamic_loading = False
    mythic_encrypts = True
    build_parameters = [
        # Add a build option which specifies whether the agent should fork in the
        # background on Linux hosts
        BuildParameter(
            name="daemonize",
            parameter_type=BuildParameterType.Boolean,
            description=(
                "Daemonize the process on Linux/Hide the console window on Windows."
            ),
            default_value=False,
            required=True,
        ),
        # Add a build option which specifies the number of initial checkin attempts
        BuildParameter(
            name="connection_retries",
            parameter_type=BuildParameterType.String,
            description=(
                "Number of times to try and reconnect if the initial checkin fails."
            ),
            default_value="1",
            verifier_regex="^[0-9]+$",
            required=True,
        ),
        # Add a build option for target architecture
        BuildParameter(
            name="architecture",
            parameter_type=BuildParameterType.ChooseOne,
            description="Target architecture.",
            default_value="x64",
            choices=["x64", "x86"],
            required=True,
        ),
        # Add a build option for working hours
        BuildParameter(
            name="working_hours",
            parameter_type=BuildParameterType.String,
            description="Working hours for the agent (use 24 hour time)",
            default_value="00:00-23:59",
            verifier_regex=r"^[0-2][0-9]:[0-5][0-9]-[0-2][0-9]:[0-5][0-9]",
            required=True,
        ),
        # Add a build option for static linking
        BuildParameter(
            name="static",
            parameter_type=BuildParameterType.Boolean,
            description=(
                "Statically link payload. (For Linux. Only works for 64 bit payloads)"
            ),
            default_value=False,
            required=True,
        ),
        # Output format
        BuildParameter(
            name="output",
            parameter_type=BuildParameterType.ChooseOne,
            description="Payload output format",
            default_value="executable",
            choices=["executable", "shared library (.dll/.so)"],
            required=True,
        ),
    ]
    # Supported C2 profiles for thanatos
    c2_profiles = ["http"]

    agent_path = pathlib.Path(os.path.dirname(__file__)) / ".." / ".."
    agent_code_path = agent_path / "agent"
    agent_icon_path = agent_path / "mythic" / "icon" / "thanatos.svg"

    # This function is called to build a new payload
    async def build(self) -> BuildResponse:
        # Setup a new build response object
        resp = BuildResponse(status=BuildStatus.Error)

        start_time = time.time()

        try:
            configured_build_parameters = self.get_payload_build_parameters()
        except ValueError as exc:
            resp.build_message(
                "Failed to build payload. Check build errors for more information"
            )
            resp.build_stderr(exc)
            return resp

        # Combine the C2 parameters with the build parameters
        payload_parameters = {
            "UUID": self.uuid,
            **self.get_c2_profile_parameters(),
            **configured_build_parameters,
        }

        # Create the build command
        build_command = [
            "env",
            " ".join(
                [
                    (
                        f"{key}='{val}'"
                        if isinstance(val, str)
                        else f"{key}='{json.dumps(val)}'"
                    )
                    for key, val in payload_parameters.items()
                ]
            ),
            "cargo build",
            f"--target {self.get_rust_triple()}",
            f"--features {self.get_c2_profile_name()}",
            "--release",
        ]

        command = " ".join(build_command)

        # Set the build stdout to the build command
        resp.build_message = str(command)

        # Make a temporary directory for building the implant
        with tempfile.TemporaryDirectory(suffix=self.uuid) as agent_build_path:
            # Copy the implant code to the temporary directory
            copytree(self.agent_code_path, agent_build_path, dirs_exist_ok=True)

            # Run the cargo command which builds the agent
            proc = await asyncio.create_subprocess_shell(
                command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=agent_build_path,
            )

            # Get the build command's stdout and stderr messages
            stdout, stderr = await proc.communicate()

            if stdout:
                resp.set_build_stdout(stdout.decode())

            if stderr:
                resp.set_build_stderr(stderr.decode())

            # Check if the build command returned an error and send that error to Mythic
            # stdout/stderr
            if proc.returncode != 0:
                resp.set_build_message(
                    "Failed to build payload. Check build errors for more information",
                )
                return resp

            resp.payload = self.get_built_payload(agent_build_path)

        elapsed = time.time() - start_time

        # Notify Mythic that the build was successful
        resp.build_message = (
            "Successfully built thanatos agent.\n\n"
            f"Payload build time: {elapsed // 60:0.0f} "
            f"{'minutes' if elapsed // 60 != 1 else 'minute'}, "
            f"{elapsed % 60:0.0f} "
            f"{'seconds' if elapsed % 60 != 1 else 'second'} "
            f"({elapsed:0.0f} total seconds)\n\n"
            "Build command:\n"
            f"{command}"
        )

        resp.status = BuildStatus.Success
        return resp

    def get_c2_profile_name(self) -> str:
        """Gets the configured C2 profile name"""
        return self.c2info[0].get_c2profile()["name"]

    def get_c2_profile_parameters(self) -> dict:
        """Gets the configured C2 profile parameters"""
        return self.c2info[0].get_parameters_dict()

    def get_payload_build_parameters(self):
        """Gets the configured build parameter values for the payload and validates them"""
        params = self.get_build_instance_values()

        # Building 32 bit statically linked payloads is not supported due to musl/openssl
        # limitations
        if (self.selected_os == SupportedOS.Linux) and (
            params["architecture"] == "x86" and params["static"]
        ):
            raise ValueError("Cannot build 32 bit statically linked payload on Linux")

        # Check working hours hour value
        working_hour = int(params["working_hours"].split(":")[0])

        if working_hour > 24:
            raise ValueError("Working hours start hour is larger than 24")

        working_hour = int(params["working_hours"].split("-")[1].split(":")[0])
        if working_hour > 24:
            raise ValueError("Working hours end hour is larger than 24")

        del params["architecture"]
        del params["static"]
        del params["output"]
        return params

    def get_rust_triple(self) -> str:
        """Gets the Rust triple from the selected build parameters

        Returns:
            Fully formed Rust triple
        """

        # Get triple architecture
        if self.get_parameter("architecture") == "x64":
            arch = "x86_64"
        else:
            arch = "i686"

        # Get triple vendor and OS type
        if self.selected_os == SupportedOS.Linux:
            vendor = "unknown"
            os_type = "linux"
        else:
            vendor = "pc"
            os_type = "windows"

        # Get triple environment type
        if self.selected_os == SupportedOS.Linux and self.get_parameter("static"):
            env_type = "musl"
        else:
            env_type = "gnu"

        return f"{arch}-{vendor}-{os_type}-{env_type}"

    def get_built_payload(self, build_path: str) -> bytes:
        """Opens the path containing the built payload and returns the raw data of it"""

        payload_path = (
            pathlib.Path(build_path) / "target" / self.get_rust_triple() / "release"
        )

        # Parse the output format for the payload
        if "executable" in self.get_parameter("output"):
            binary_name = "thanatos" + (
                ".exe" if self.selected_os == SupportedOS.Windows else ""
            )

        elif "shared library" in self.get_parameter("output"):
            binary_name = (
                "libthanatos.so"
                if self.selected_os == SupportedOS.Linux
                else "thanatos.dll"
            )

        payload_path = payload_path / binary_name
        with open(payload_path, "rb") as f:
            return f.read()
