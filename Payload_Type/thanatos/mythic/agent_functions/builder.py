from mythic_payloadtype_container.PayloadBuilder import (
    PayloadType,
    SupportedOS,
    BuildParameter,
    BuildParameterType,
    BuildResponse,
    BuildStatus,
)
from mythic_payloadtype_container.MythicCommandBase import BrowserScript

import asyncio
import os
import donut
from shutil import copytree
import tempfile
import json


# Class defining information about the Thanatos payload
class Thanatos(PayloadType):
    name = "thanatos"  # Name of the payload
    file_extension = "exe"  # default file extension to use when creating payloads
    author = "@M_alphaaa, 0xdab0"  # authors

    # Platforms that thanatos supports
    supported_os = [
        SupportedOS.Windows,
        SupportedOS.Linux,
    ]
    wrapper = False
    wrapped_payloads = []
    note = "Linux and Windows agent written in Rust"  # Note about the payload displayed in Mythic
    supports_dynamic_loading = False  # Payload does not support dynamic loading
    mythic_encrypts = True
    build_parameters = [
        # Add a build option which specifies whether the agent should fork in the
        # background on Linux hosts
        BuildParameter(
            name="daemonize",
            parameter_type=BuildParameterType.Boolean,
            description="Daemonize the process on Linux/Hide the console window on Windows.",
            default_value=False,
            required=True,
        ),
        # Add a build option which specifies the number of initial checkin attempts
        BuildParameter(
            name="connection_retries",
            parameter_type=BuildParameterType.String,
            description="Number of times to try and reconnect if the initial checkin fails.",
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
            description="Statically link payload. (For Linux)",
            default_value=False,
            required=True,
        ),
        # Output format
        BuildParameter(
            name="output",
            parameter_type=BuildParameterType.ChooseOne,
            description="Payload output format (shellcode only works for Windows)",
            default_value="executable",
            choices=["executable", "shared library (.dll/.so)", "shellcode"],
            required=True,
        ),
    ]
    # Supported C2 profiles for thanatos
    c2_profiles = ["http"]

    # This function is called to build a new payload
    async def build(self) -> BuildResponse:
        # Setup a new build response object
        resp = BuildResponse(status=BuildStatus.Error)

        try:
            # Make a temporary directory for building the implant
            agent_build_path = tempfile.TemporaryDirectory(suffix=self.uuid)

            # Copy the implant code to the temporary directory
            copytree(self.agent_code_path, agent_build_path.name, dirs_exist_ok=True)

            # Get the C2 profile information
            c2 = self.c2info[0]
            profile = c2.get_c2profile()["name"]
            if profile not in self.c2_profiles:
                resp.build_message = "Invalid C2 profile name specified"
                return resp

            # Check if building for shellcode on Linux and fail since it is not
            # implemented
            if (
                self.get_parameter("output") == "shellcode"
                and self.selected_os == SupportedOS.Linux
            ):
                raise Exception("Cannot build shellcode for Linux platform")

            # Get the architecture from the build parameter
            if self.get_parameter("architecture") == "x64":
                arch = "x86_64"
            else:
                arch = "i686"

            # Add the C2 profile to the compile flags
            rustflags = f"--cfg {profile} "

            # Check for static linking
            abi = "gnu"
            if self.selected_os == SupportedOS.Linux:
                if self.get_parameter("static"):
                    rustflags += "-C target-feature=+crt-static"
                    abi = "musl"

            # Get the target OS to compile for from the selected OS in Mythic
            target_os = (
                f"{arch}-unknown-linux-{abi}"
                if self.selected_os == SupportedOS.Linux
                else f"{arch}-pc-windows-gnu"
            )

            # Combine the C2 parameters with the build parameters
            c2_params = c2.get_parameters_dict()
            c2_params["UUID"] = self.uuid

            if self.get_parameter("output") == "shellcode":
                c2_params["daemonize"] = "false"
            else:
                c2_params["daemonize"] = str(self.get_parameter("daemonize"))

            c2_params["connection_retries"] = self.get_parameter("connection_retries")
            c2_params["working_hours"] = self.get_parameter("working_hours")

            # Start formulating the command to build the agent
            command = "env "

            # Set up openssl environment variables
            openssl_env = "OPENSSL_STATIC=yes "
            if arch == "x86_64":
                openssl_env += "OPENSSL_LIB_DIR=/usr/lib64 "
            else:
                openssl_env += "OPENSSL_LIB_DIR=/usr/lib "

            openssl_env += "OPENSSL_INCLUDE_DIR=/usr/include "

            command += openssl_env

            # Add any rustflags if they exist
            if rustflags:
                command += f'RUSTFLAGS="{rustflags}" '

            # Loop through each C2/build parameter creating environment variable
            # key/values for each option
            for key, val in c2_params.items():
                if isinstance(val, str):
                    command += f"{key}='{val}' "
                else:
                    v = json.dumps(val)
                    command += f"{key}='{v}' "

            # Finish off the cargo command used for building the agent
            command += f"cargo build --target {target_os} --release"

            # Copy any prebuilt dependencies if they exist
            deps_suffix = "_static" if self.get_parameter("static") == "yes" else ""

            deps_path = f"/opt/{target_os}{deps_suffix}"
            if os.path.exists(deps_path):
                copytree(
                    f"{deps_path}",
                    f"{agent_build_path.name}/target",
                    dirs_exist_ok=True,
                )

            # Set the build stdout to the build command invocation
            resp.build_message = str(command)

            # Run the cargo command which builds the agent
            proc = await asyncio.create_subprocess_shell(
                command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=agent_build_path.name,
            )

            # Grab stdout/stderr
            stdout, stderr = await proc.communicate()

            # Check if the build command returned an error and send that error to Mythic
            # stdout/stderr
            if proc.returncode != 0:
                resp.set_build_stdout(stdout.decode())
                resp.set_build_stderr(stderr.decode())
                raise Exception("Failed to build payload. Check Build Errors")

            # Copy any dependencies that were compiled
            built_path = f"{agent_build_path.name}/target"
            if os.path.exists(built_path):
                copytree(f"{built_path}", f"{deps_path}", dirs_exist_ok=True)

            # Check if there is anything on stdout/stderr and forward to Mythic
            if stdout:
                resp.set_build_stdout(f"{command}\n\n{stdout.decode()}")
            if stderr:
                resp.set_build_stderr(stdout.decode())

            # Parse the output format for the payload
            if "executable" in self.get_parameter("output"):
                # Set the payload output to the built executable
                target_name = (
                    "thanatos"
                    if self.selected_os == SupportedOS.Linux
                    else "thanatos.exe"
                )
                payload_path = (
                    f"{agent_build_path.name}/target/{target_os}/release/{target_name}"
                )
            elif "shared library" in self.get_parameter("output"):
                # Set the payload output to the build shared library
                target_name = (
                    "libthanatos.so"
                    if self.selected_os == SupportedOS.Linux
                    else "thanatos.dll"
                )
                payload_path = (
                    f"{agent_build_path.name}/target/{target_os}/release/{target_name}"
                )
            elif "shellcode" in self.get_parameter("output"):
                # Grab the dll if compiling to shellcode
                target_name = "thanatos.dll"
                payload_path = (
                    f"{agent_build_path.name}/target/{target_os}/release/{target_name}"
                )

            # Convert the payload to shellcode if the output format was shellcode
            if (
                self.get_parameter("output") == "shellcode"
                and self.selected_os == SupportedOS.Windows
            ):
                # Create shellcode using donut
                resp.payload = donut.create(file=payload_path)
            else:
                resp.payload = open(payload_path, "rb").read()

            # Notify Mythic that the build was successful
            resp.set_build_message("Successfully built thanatos agent.")
            resp.build_message += "\n"
            resp.build_message += str(command)
            resp.status = BuildStatus.Success
        except Exception as e:
            # Mythic failed to build the payload
            import traceback
            import sys

            # Return the python exception to the Mythic build message
            exc_type, exc_value, exc_traceback = sys.exc_info()
            resp.build_stderr += f"Error building payload: {e} traceback: " + repr(
                traceback.format_exception(exc_type, exc_value, exc_traceback)
            )

        return resp
