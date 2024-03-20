import json
from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
    SupportedOS,
    BrowserScript,
    PTTaskMessageAllData,
    PTTaskCreateTaskingMessageResponse,
)

# TODO: Refactor implementation


class DownloadArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="file",
                type=ParameterType.String,
                description="File to download.",
                display_name="File to download.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                temp_json = json.loads(self.command_line)
                if "path" in temp_json:
                    self.set_arg("file", f"{temp_json['path']}/{temp_json['file']}")
                else:
                    self.set_arg("file", temp_json["file"])
            else:
                self.set_arg("file", self.command_line)

    async def parse_dictionary(self, dictionary_arguments):
        if "path" in dictionary_arguments:
            dictionary_arguments["file"] = (
                f"{dictionary_arguments['path']}, {dictionary_arguments['file']}"
            )
        self.load_args_from_dictionary(dictionary_arguments)


class DownloadCommand(CommandBase):
    cmd = "download"
    needs_admin = False
    help_cmd = "download [file]"
    description = "Download a file from the target."
    version = 2
    is_download_file = True
    author = "@M_alphaaa"
    argument_class = DownloadArguments
    attackmapping = ["T1020", "T1030", "T1041"]
    supported_ui_features = ["file_browser:download"]
    browser_script = BrowserScript(
        script_name="download", author="@M_alphaaa", for_new_ui=True
    )
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, task_data: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        return PTTaskCreateTaskingMessageResponse(
            TaskID=task_data.Task.ID,
            DisplayParams=str(task_data.args.get_arg("file")),
            Success=True,
        )
