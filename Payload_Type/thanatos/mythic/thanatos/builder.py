import asyncio
import base64
import json
import os
import shutil
import tempfile
import time
from datetime import datetime
from pathlib import Path

from mythic_container.MythicGoRPC import (
    MythicRPCPayloadUpdateBuildStepMessage,
    SendMythicRPCPayloadUpdatebuildStep,
)
from mythic_container.PayloadBuilder import (
    BuildParameter,
    BuildParameterType,
    BuildResponse,
    BuildStatus,
    BuildStep,
    PayloadType,
    SupportedOS,
)


# Class defining information about the Thanatos payload
class Thanatos(PayloadType):
    name = "thanatos"  # Name of the payload
    file_extension = ""  # default file extension to use when creating payloads
    author = "@M_alphaaa"  # authors

    supported_os = [
        SupportedOS.Windows,
        SupportedOS.Linux,
    ]
    wrapper = False
    wrapped_payloads = []
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
            description="Number of times to try and reconnect on failed callbacks",
            default_value="10",
            verifier_regex="^[0-9]+$",
            required=True,
        ),
        # Add a build option for target architecture
        BuildParameter(
            name="architecture",
            parameter_type=BuildParameterType.ChooseOne,
            description="Target architecture",
            default_value="x86_64",
            choices=["x86_64", "i686"],
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
            choices=["executable", "shared library"],
            required=True,
        ),
    ]
    # Supported C2 profiles for thanatos
    c2_profiles = ["http"]

    build_steps = [
        BuildStep("Validating configuration", "Validating payload configuration"),
        BuildStep(
            "Compiling agent and dependencies",
            "Building the payload and its dependencies",
        ),
        BuildStep(
            "Compiling agent with built dependencies",
            "Building the payload with already built dependencies",
        ),
    ]

    payload_base = Path(os.path.dirname(__file__)) / ".." / ".."
    agent_path = payload_base / "agent"
    agent_code_path = agent_path
    agent_browserscript_path = payload_base / "mythic" / "browserscripts"
    agent_icon_path = payload_base / "mythic" / "icon" / "thanatos.svg"
    copy_lock = asyncio.Lock()

    # This function is called to build a new payload
    async def build(self) -> BuildResponse:
        resp = BuildResponse(status=BuildStatus.Error)

        payload_parameters = {
            "UUID": self.uuid,
            **self.get_build_instance_values(),
        }

        arch = payload_parameters["architecture"]
        if self.selected_os == SupportedOS.Linux:
            vendor = "unknown"
            os = "linux"

            if payload_parameters["static"]:
                if arch == "i686":
                    SendMythicRPCPayloadUpdatebuildStep(
                        MythicRPCPayloadUpdateBuildStepMessage(
                            self.uuid,
                            StepName="Validating configuration",
                            StepSuccess=False,
                            StepStderr="Cannot build a 32bit statically linked payload",
                        )
                    )
                    return resp

                payload_parameters["RUSTFLAGS"] = "-C target-feature=+crt-static"
                environment = "musl"
            else:
                environment = "gnu"
        else:
            payload_parameters["RUSTFLAGS"] = "-C target-feature=+crt-static"
            vendor = "pc"
            os = "windows"
            environment = "gnu"

        triple = f"{arch}-{vendor}-{os}-{environment}"

        working_start = time.strptime(
            payload_parameters["working_hours"].split("-")[0], "%H:%M"
        )
        payload_parameters["working_start"] = (
            working_start.tm_hour * 3600 + working_start.tm_min * 60
        )

        working_end = time.strptime(
            payload_parameters["working_hours"].split("-")[1], "%H:%M"
        )
        payload_parameters["working_end"] = (
            working_end.tm_hour * 3600 + working_end.tm_min * 60
        )

        if working_end <= working_start:
            SendMythicRPCPayloadUpdatebuildStep(
                MythicRPCPayloadUpdateBuildStepMessage(
                    self.uuid,
                    StepName="Validating configuration",
                    StepSuccess=False,
                    StepStderr="Working hours end portion is before the working hours start portion",
                )
            )
            return resp

        for profile in self.c2info:
            name = profile.get_c2profile()["name"]
            parameters = profile.get_parameters_dict()
            for key, val in parameters.items():
                if key == "killdate":
                    val = int(datetime.fromisoformat(val).timestamp())
                elif key == "AESPSK":
                    if val["value"] == "aes256_hmac":
                        val = val["enc_key"]
                    else:
                        val = ""
                elif isinstance(val, dict):
                    val = base64.b64encode(json.dumps(val))
                payload_parameters[name + "_" + key] = str(val)

        SendMythicRPCPayloadUpdatebuildStep(
            MythicRPCPayloadUpdateBuildStepMessage(
                self.uuid,
                StepName="Validating configuration",
                StepSuccess=True,
                StepStdout=json.dumps(payload_parameters, indent=2),
            )
        )

        if payload_parameters["output"] == "executable":
            variant = "bin"
        else:
            variant = "cdylib"

        with tempfile.TemporaryDirectory(prefix="thanatos_", suffix=self.uuid) as tmpdir:
            async with self.copy_lock:
                shutil.copytree(self.agent_code_path, tmpdir.name)

            dependencies_prebuilt = False
            if Path(tmpdir.name + "/target/" + triple).exists():
                SendMythicRPCPayloadUpdatebuildStep(
                    MythicRPCPayloadUpdateBuildStepMessage(
                        self.uuid,
                        StepName="Compiling agent and dependencies",
                        StepSuccess=True,
                        StepSkip=True,
                    )
                )
                dependencies_prebuilt = True

            proc = await asyncio.create_subprocess_exec(
                "/usr/local/bin/cargo",
                "build",
                "--target",
                triple,
                "--release",
                "-p",
                f"thanatos_{variant}",
                env=payload_parameters,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )

            stdout, stderr = await proc.communicate()
            if proc.returncode != 0:
                success = False
            else:
                success = True

            if dependencies_prebuilt:
                SendMythicRPCPayloadUpdatebuildStep(
                    MythicRPCPayloadUpdateBuildStepMessage(
                        self.uuid,
                        StepName="Compiling agent with built dependencies",
                        StepSuccess=success,
                        StepStdout=stdout,
                        StepStderr=stderr,
                    )
                )
            else:
                SendMythicRPCPayloadUpdatebuildStep(
                    MythicRPCPayloadUpdateBuildStepMessage(
                        self.uuid,
                        StepName="Compiling agent and dependencies",
                        StepSuccess=success,
                        StepStdout=stdout,
                        StepStderr=stderr,
                    )
                )
                SendMythicRPCPayloadUpdatebuildStep(
                    MythicRPCPayloadUpdateBuildStepMessage(
                        self.uuid,
                        StepName="Compiling agent with built dependencies",
                        StepSuccess=True,
                        StepSkip=True,
                    )
                )

            if not success:
                return resp

            if "linux" in triple and payload_parameters["output"] == "executable":
                outfile = f"thanatos_{variant}"
            elif "linux" in triple and payload_parameters["output"] == "shared library":
                outfile = f"libthanatos_{variant}.so"
            elif "windows" in triple and payload_parameters["output"] == "executable":
                outfile = f"thanatos_{variant}.exe"
            elif "windows" in triple and payload_parameters["output"] == "shared library":
                outfile = f"thanatos_{variant}.dll"

            with open(tmpdir.name + "/target/" + triple + f"/{outfile}", "rb") as f:
                resp.set_payload(f.read())

            async with self.copy_lock:
                shutil.copytree(tmpdir.name, self.agent_code_path)

        resp.set_status(BuildStatus.Success)
        return resp
