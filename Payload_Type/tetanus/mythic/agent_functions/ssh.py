from mythic_payloadtype_container.MythicRPC import MythicRPC
from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    MythicTask,
    AgentResponse,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
    MythicStatus,
    BrowserScript,
)
import json
import sys
import base64


class SshArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="credentials",
                type=ParameterType.Credential_JSON,
                description="Credentials to use",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Execute",
                        ui_position=1,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Download",
                        ui_position=1,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Upload",
                        ui_position=1,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="List",
                        ui_position=1,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Cat",
                        ui_position=1,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Remove",
                        ui_position=1,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="agent",
                display_name="use ssh agent",
                type=ParameterType.Boolean,
                description="Use the ssh agent for auth",
                default_value=False,
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Execute",
                        ui_position=2,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Download",
                        ui_position=2,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Upload",
                        ui_position=2,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="List",
                        ui_position=2,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Cat",
                        ui_position=2,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Remove",
                        ui_position=2,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="host",
                type=ParameterType.String,
                description="Hostname or IP address of remote machine",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Execute",
                        ui_position=3,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Download",
                        ui_position=3,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Upload",
                        ui_position=3,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="List",
                        ui_position=3,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Cat",
                        ui_position=3,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Remove",
                        ui_position=3,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="port",
                type=ParameterType.Number,
                description="Port number for ssh connection",
                default_value=22,
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Execute",
                        ui_position=4,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Download",
                        ui_position=4,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Upload",
                        ui_position=4,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="List",
                        ui_position=4,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Cat",
                        ui_position=4,
                        required=True,
                    ),
                    ParameterGroupInfo(
                        group_name="Remove",
                        ui_position=4,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="exec",
                type=ParameterType.String,
                description="Command to execute on the system",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Execute",
                        ui_position=6,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="cat",
                display_name="file",
                type=ParameterType.String,
                description="File to cat",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Cat",
                        ui_position=6,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="rm",
                display_name="file",
                type=ParameterType.String,
                description="File or directory to remove",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Remove",
                        ui_position=6,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="download",
                display_name="download path",
                type=ParameterType.String,
                description="File to download from the remote system",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Download",
                        ui_position=6,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="upload",
                display_name="file",
                type=ParameterType.File,
                description="File to upload",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Upload",
                        ui_position=6,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="upload_path",
                cli_name="path",
                display_name="upload path",
                type=ParameterType.String,
                description="Absolute path to upload the file to",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Upload",
                        ui_position=7,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="mode",
                cli_name="mode",
                display_name="mode (octal)",
                type=ParameterType.Number,
                description="Octal permissions for the uploaded file",
                default_value=644,
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="Upload",
                        ui_position=8,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="list",
                cli_name="ls",
                display_name="list directory",
                type=ParameterType.String,
                description="Path to get directory listing of",
                parameter_group_info=[
                    ParameterGroupInfo(
                        group_name="List",
                        ui_position=6,
                        required=True,
                    ),
                ],
            ),
        ]

    async def parse_arguments(self):
        self.load_args_from_json_string(self.command_line)

    async def parse_dictionary(self, dictionary_arguments: dict):
        self.load_args_from_dictionary(dictionary_arguments)


class SshCommand(CommandBase):
    cmd = "ssh"
    needs_admin = False
    help_cmd = "ssh [-exec <command>] [-upload <file>] [-download <path>] [-ls <path>] [-cat <file>]"
    description = "Use ssh to upload/download/cat files, get directory listings and execute commands"
    version = 1
    is_upload_file = True
    author = "@M_alphaaa"
    attackmapping = ["T1021.004"]
    supported_ui_features = ["ssh"]
    argument_class = SshArguments
    browser_script = BrowserScript(
        script_name="ssh", author="@M_alphaaa", for_new_ui=True
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        creds = task.args.get_arg("credentials")
        user = creds["account"]
        host = task.args.get_arg("host")
        auth_type = creds["type"]

        if task.callback.host == "Windows" and auth_type == "key":
            raise Exception("Cannot use key auth on Windows hosts")

        if path := task.args.get_arg("download"):
            task.display_params = f"{user}@{host} -download {path}"
        elif path := task.args.get_arg("upload"):
            try:
                mode = str(task.args.get_arg("mode"))
                mode_int = int(mode, 8)
                task.args.set_arg("mode", mode_int)
            except Exception:
                raise Exception("Mode not in octal format")

            try:
                original_file_name = json.loads(task.original_params)["upload"]
                file_resp = await MythicRPC().execute(
                    "create_file",
                    task_id=task.id,
                    file=base64.b64encode(path.encode()).decode(),
                    saved_file_name=original_file_name,
                    delete_after_fetch=True,
                )
                if file_resp.status == MythicStatus.Success:
                    task.args.add_arg("file", file_resp.response["agent_file_id"])
                else:
                    raise Exception("Error from Mythic: " + str(file_resp.error))
            except Exception as e:
                raise Exception(
                    "Error from Mythic: " + str(sys.exc_info()[-1].tb_lineno) + str(e)
                )

            task.display_params = (
                f"{user}@{host} -upload '{original_file_name}' to"
                f" {task.args.get_arg('upload_path')}"
            )

        elif cmd := task.args.get_arg("exec"):
            task.display_params = f"{user}@{host} -exec {cmd}"
        elif path := task.args.get_arg("list"):
            task.display_params = f"{user}@{host} -ls {path}"
        elif path := task.args.get_arg("cat"):
            task.display_params = f"{user}@{host} -cat {path}"
        elif path := task.args.get_arg("rm"):
            task.display_params = f"{user}@{host} -rm {path}"

        return task

    async def process_response(self, response: AgentResponse):
        pass
