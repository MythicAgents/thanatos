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
    SendMythicRPCFileSearch,
    MythicRPCFileSearchMessage,
)


class UploadArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="file",
                type=ParameterType.File,
                description="File to upload.",
                display_name="File to upload.",
                parameter_group_info=[
                    ParameterGroupInfo(
                        ui_position=1,
                        required=True,
                    ),
                ],
            ),
            CommandParameter(
                name="path",
                cli_name="path",
                type=ParameterType.String,
                description="Path where to upload the file including the file name.",
                display_name="Path where to upload the file including the file name.",
                parameter_group_info=[
                    ParameterGroupInfo(
                        ui_position=2,
                        required=True,
                    ),
                ],
            ),
        ]

    async def parse_arguments(self):
        self.load_args_from_json_string(self.command_line)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class UploadCommand(CommandBase):
    cmd = "upload"
    needs_admin = False
    help_cmd = "upload"
    description = (
        "Upload a file to the target machine by selecting a file from your computer."
    )
    version = 1
    is_file_upload = True
    supported_ui_features = ["file_browser:upload"]
    author = "@M_alphaaa"
    attackmapping = ["T1030", "T1105", "T1132"]
    argument_class = UploadArguments
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        try:
            file_id = task.args.get_arg("file")
            resp = await SendMythicRPCFileSearch(
                MythicRPCFileSearchMessage(
                    TaskID=task.id,
                    AgentFileId=file_id,
                )
            )

            if not resp.Success:
                raise Exception(resp.error)

            file_name = resp.Files[0].Filename

            if len(task.args.get_arg("path")) == 0:
                task.args.add_arg("path", file_name)
            elif task.args.get_arg("path")[-1] == "/":
                task.args.add_arg("path", task.args.get_arg("path") + file_name)

            task.display_params = f"{file_name} to {task.args.get_arg('path')}"
            return task
        except Exception as e:
            raise Exception(
                f"Error from line {str(sys.exc_info()[-1].tb_lineno)}: {str(e)}"
            )

    async def process_response(
        self, task: PTTaskMessageAllData, response: str
    ) -> PTTaskProcessResponseMessageResponse:
        pass
