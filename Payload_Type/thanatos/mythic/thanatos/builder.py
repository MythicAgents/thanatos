import asyncio
import base64
import copy
import json
import os
import pathlib
from datetime import datetime, timezone
from typing import List

from mythic_container.MythicGoRPC import (
    MythicRPCPayloadUpdateBuildStepMessage,
    SendMythicRPCPayloadUpdatebuildStep)
from mythic_container.PayloadBuilder import (BuildParameter,
                                             BuildParameterType, BuildResponse,
                                             BuildStatus, BuildStep,
                                             PayloadType, SupportedOS)


def format_time(ts: float) -> str:
    ts = int(ts)
    minutes = int(ts // 60)
    seconds = int(ts % 60)

    return (
        f"{minutes} minute{'s' if minutes != 1 else ''}"
        " "
        f"{seconds} second{'s' if seconds != 1 else ''}"
        " "
        f"({ts} total second{'s' if ts != 0 else ''})"
    )


configure_lock = asyncio.Lock()
build_lock = asyncio.Lock()


# Class defining information about the Thanatos payload
class Thanatos(PayloadType):
    name = "thanatos"  # Name of the payload
    file_extension = ""  # default file extension to use when creating payloads
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
                "Daemonize the process on Linux/Hide the console window on Windows"
            ),
            default_value=True,
            required=True,
        ),
        # Add a build option which specifies the number of initial checkin attempts
        BuildParameter(
            name="connection_retries",
            parameter_type=BuildParameterType.String,
            description=(
                "Number of times to try and reconnect on failed check-ins."
            ),
            default_value="10",
            verifier_regex="^[0-9]+$",
            required=True,
        ),
        BuildParameter(
            name="tlsuntrusted",
            parameter_type=BuildParameterType.Boolean,
            description="Allow https connections to untrusted TLS certificates",
            default_value=False,
            required=True,
        ),
        # Add a build option for target architecture
        BuildParameter(
            name="architecture",
            parameter_type=BuildParameterType.ChooseOne,
            description="Target architecture",
            default_value="x86_64",
            choices=["x86_64", "x86"],
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
        # System proxy
        BuildParameter(
            name="systemproxy",
            parameter_type=BuildParameterType.Boolean,
            description="Use the system's configured http proxy settings.",
            default_value=False,
            required=True,
        ),
        # Add a build option for static linking
        BuildParameter(
            name="static",
            parameter_type=BuildParameterType.Boolean,
            description="Statically link payload (Only Linux 64 bit payloads)",
            default_value=False,
            required=True,
        ),
        # Output format
        BuildParameter(
            name="output",
            parameter_type=BuildParameterType.ChooseOne,
            description="Payload output format",
            default_value="executable",
            choices=["executable", "cdylib (.dll/.so)"],
            required=True,
        ),
    ]
    # Supported C2 profiles for thanatos
    c2_profiles = ["http"]

    build_steps = [
        BuildStep("Validating configuration", "Validating payload configuration"),
        BuildStep(
            "Waiting for previous builds to finish",
            "Waiting for previous payloads to finish building",
        ),
        BuildStep("Compiling agent", "Compiling the payload"),
    ]

    agent_path = pathlib.Path(os.path.dirname(__file__)) / ".." / ".."
    agent_code_path = agent_path / "agent"
    agent_browserscript_path = agent_path / "mythic" / "browserscripts"
    agent_icon_path = agent_path / "mythic" / "icon" / "thanatos.svg"

    # This function is called to build a new payload
    async def build(self) -> BuildResponse:
        async with configure_lock:
            payload_uuid = self.uuid
            build_os = self.selected_os
            try:
                parameters = self.get_all_parameters()
            except (ValueError, IndexError) as exc:
                await SendMythicRPCPayloadUpdatebuildStep(
                    MythicRPCPayloadUpdateBuildStepMessage(
                        PayloadUUID=payload_uuid,
                        StepName="Validating configuration",
                        StepSuccess=False,
                        StepStdout="Configuration is invalid. Check errors for more information",
                        StepStderr=str(exc),
                    )
                )

                return BuildResponse(BuildStatus.Error)

        # Combine the C2 parameters with the build parameters
        payload_parameters = {
            "UUID": self.uuid,
            **self.filter_build_options(parameters),
        }

        # Create the build command
        command = [
            "env",
            " ".join(f"{key}='{val}'" for key, val in payload_parameters.items()),
            "cargo build",
            f"-p {'thanatos-bin' if 'executable' in parameters['output'] else 'thanatos-lib'}",
            f"--target {self.get_rust_triple(build_os, parameters)}",
            "--release",
        ]

        if features := self.derive_feature_flags(parameters):
            command.append(f"--features {','.join(features)}")

        command = " ".join(command)

        await SendMythicRPCPayloadUpdatebuildStep(
            MythicRPCPayloadUpdateBuildStepMessage(
                PayloadUUID=payload_uuid,
                StepName="Validating configuration",
                StepSuccess=True,
                StepStdout=(
                    "Payload configuration is valid.\n"
                    "\n"
                    "Configuration JSON:\n"
                    f"{json.dumps(payload_parameters, indent=2)}"
                    "\n"
                    "\n"
                    "Build command:\n"
                    f"{command}"
                ),
            )
        )

        build_waited = build_lock.locked()
        async with build_lock:
            await SendMythicRPCPayloadUpdatebuildStep(
                MythicRPCPayloadUpdateBuildStepMessage(
                    PayloadUUID=payload_uuid,
                    StepName="Waiting for previous builds to finish",
                    StepSuccess=True,
                    StepSkip=not build_waited,
                )
            )

            return await self.run_payload_build(
                payload_uuid, command, build_os, parameters
            )

    def get_payload_build_parameters(self):
        """Gets the configured build parameter values for the payload and validates them"""
        params = self.get_build_instance_values()

        # Building 32 bit statically linked payloads is not supported due to musl/openssl
        # limitations
        if (self.selected_os == SupportedOS.Linux) and (
            params["architecture"] == "x86" and params["static"]
        ):
            raise ValueError("Cannot build 32 bit statically linked payload on Linux")

        working_hours = params["working_hours"]
        del params["working_hours"]

        params["working_start"] = str(
            self.parse_working_hours(working_hours.split("-")[0])
        )
        params["working_end"] = str(self.parse_working_hours(working_hours.split("-")[1]))

        if connection_retries := params["connection_retries"]:
            arch = params["architecture"]
            if arch == "x86_64":
                bytewidth = 8
            else:
                bytewidth = 4

            if int(connection_retries) > (2**(bytewidth * 8)) - 1:
                raise ValueError("Connection retries is too large for the target architecture")

        return params

    def get_profile_parameters(self) -> dict:
        params = self.c2info[0].get_parameters_dict()
        params["killdate"] = str(
            int(
                datetime.fromisoformat(params["killdate"])
                .replace(tzinfo=timezone.utc)
                .timestamp()
            )
        )

        if headers := params["headers"]:
            params["headers"] = base64.b64encode(json.dumps(headers).encode()).decode()
        else:
            del params["headers"]

        if aes_key := params["AESPSK"]["enc_key"]:
            params["AESKEY"] = aes_key
        del params["AESPSK"]

        if jitter := params["callback_jitter"]:
            if int(jitter) < 0 or int(jitter) >= 100:
                raise ValueError("Callback jitter cannot be >=100%")

        if self.get_parameter("systemproxy") == "false":
            if proxy_host := params["proxy_host"]:
                proxy_info = {"host": proxy_host}

                if port := params["proxy_port"]:
                    proxy_info["port"] = port
                else:
                    if proxy_host.startswith("https://"):
                        proxy_info["port"] = 443
                    else:
                        proxy_info["port"] = 80

                if user := params["proxy_user"]:
                    proxy_info["user"] = user

                if proxy_pass := params["proxy_pass"]:
                    proxy_info["password"] = proxy_pass

                params["proxy_info"] = base64.b64encode(
                    json.dumps(proxy_info).encode()
                ).decode()

        del params["proxy_host"]
        del params["proxy_port"]
        del params["proxy_user"]
        del params["proxy_pass"]
        return params

    def get_all_parameters(self) -> dict:
        return {**self.get_payload_build_parameters(), **self.get_profile_parameters()}

    def parse_working_hours(self, working_hours: str) -> str:
        # Check working hours hour value
        hour = int(working_hours.split(":")[0])
        minute = int(working_hours.split(":")[1])

        if hour > 24:
            raise ValueError("Working hours hour portion is larger than 24")

        if minute < 0 or minute > 59:
            raise ValueError("Working hours minute portion is out of range")

        return f"{hour:02d}:{minute:02d}"

    def derive_feature_flags(self, parameters: dict) -> List[str]:
        features = []

        if parameters["daemonize"]:
            features.append("daemonize")

        if parameters["tlsuntrusted"]:
            features.append("tlsuntrusted")

        if parameters["systemproxy"]:
            features.append("systemproxy")

        if parameters["encrypted_exchange_check"]:
            features.append("eke")

        return features

    def filter_build_options(self, parameters: dict) -> dict:
        params = copy.deepcopy(parameters)
        del params["daemonize"]
        del params["tlsuntrusted"]
        del params["systemproxy"]
        del params["encrypted_exchange_check"]
        del params["architecture"]
        del params["static"]
        del params["output"]

        return params

    def get_rust_triple(self, build_os, build_parameters: dict) -> str:
        """Gets the Rust triple from the selected build parameters

        Returns:
            Fully formed Rust triple
        """

        # Get triple architecture
        arch = build_parameters["architecture"]
        if arch == "x86":
            arch = "i686"

        if arch not in ("x86_64", "i686"):
            raise ValueError(f"Unknown architecture '{arch}'")

        # Get triple vendor and OS type
        if build_os == SupportedOS.Linux:
            vendor = "unknown"
            os_type = "linux"
        else:
            vendor = "pc"
            os_type = "windows"

        # Get triple environment type
        if build_os == SupportedOS.Linux and build_parameters["static"]:
            env_type = "musl"
        else:
            env_type = "gnu"

        return f"{arch}-{vendor}-{os_type}-{env_type}"

    async def run_payload_build(
        self, payload_uuid: str, command: str, build_os, parameters: dict
    ) -> BuildResponse:
        # Run the cargo command which builds the agent
        proc = await asyncio.create_subprocess_shell(
            command,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            cwd=self.agent_code_path,
        )

        # Get the build command's stdout and stderr messages
        stdout, stderr = await proc.communicate()

        if proc.returncode != 0:
            await SendMythicRPCPayloadUpdatebuildStep(
                MythicRPCPayloadUpdateBuildStepMessage(
                    PayloadUUID=payload_uuid,
                    StepName="Compiling agent",
                    StepSuccess=False,
                    StepStdout=stdout.decode(),
                    StepStderr=stderr.decode(),
                )
            )
            return BuildResponse(BuildStatus.Error)

        await SendMythicRPCPayloadUpdatebuildStep(
            MythicRPCPayloadUpdateBuildStepMessage(
                PayloadUUID=payload_uuid,
                StepName="Compiling agent",
                StepSuccess=True,
                StepStdout=stdout.decode(),
                StepStderr=stderr.decode(),
            )
        )

        payload = self.get_built_payload(build_os, parameters)

        return BuildResponse(
            status=BuildStatus.Success,
            payload=payload,
            build_message="Successfully built thanatos agent.\n",
        )

    def get_built_payload(self, build_os, parameters: dict) -> bytes:
        """Opens the path containing the built payload and returns the raw data of it"""

        payload_path = (
            self.agent_code_path
            / "target"
            / self.get_rust_triple(build_os, parameters)
            / "release"
        )

        binary_name = "thanatos"

        # Parse the output format for the payload
        if "executable" in parameters["output"]:
            binary_name += "-bin"
            if build_os == SupportedOS.Windows:
                binary_name += ".exe"

        elif "cdylib" in parameters["output"]:
            if build_os == SupportedOS.Linux:
                binary_name = "libthanatos-lib.so"
            else:
                binary_name += "-lib.dll"

        payload_path = payload_path / binary_name
        with open(payload_path, "rb") as f:
            return f.read()
