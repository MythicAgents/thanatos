from mythic_container.MythicCommandBase import (
    CommandAttributes,
    CommandBase,
    CommandParameter,
    ParameterGroupInfo,
    ParameterType,
    PTTaskCreateTaskingMessageResponse,
    PTTaskMessageAllData,
    SupportedOS,
    TaskArguments,
)
from mythic_container.MythicGoRPC import (
    MythicRPCFileSearchMessage,
    SendMythicRPCFileSearch,
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

    async def create_go_tasking(
        self, taskData: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        file_id = taskData.args.get_arg("file")
        search_resp = await SendMythicRPCFileSearch(
            MythicRPCFileSearchMessage(
                TaskID=taskData.id,
                AgentFileId=file_id,
            )
        )

        if not search_resp.Success:
            raise Exception(search_resp.error)

        file_name = search_resp.Files[0].Filename

        if len(taskData.args.get_arg("path")) == 0:
            taskData.args.add_arg("path", file_name)
        elif taskData.args.get_arg("path")[-1] == "/":
            taskData.args.add_arg("path", taskData.args.get_arg("path") + file_name)

        return PTTaskCreateTaskingMessageResponse(
            TaskID=taskData.Task.ID,
            DisplayParams=f"{file_name} to {taskData.args.get_arg('path')}",
        )
