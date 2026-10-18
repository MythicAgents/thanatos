import json

from mythic_container.MythicCommandBase import (
    BrowserScript,
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


class LsArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="path",
                type=ParameterType.String,
                description="Path to get the listing from.",
                display_name="Path to get the listing from.",
                default_value=".",
                parameter_group_info=[ParameterGroupInfo(ui_position=1, required=False)],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            self.add_arg("file_browser", False, type=ParameterType.Boolean)
        if len(self.command_line) > 0:
            try:
                tmp_json = json.loads(self.command_line)
                self.command_line = tmp_json["path"]
                self.add_arg("file_browser", True, type=ParameterType.Boolean)
            except Exception:
                pass
            self.set_arg("path", self.command_line)
        else:
            self.set_arg("path", ".")

    async def parse_dictionary(self, dictionary_arguments):
        if "file" in dictionary_arguments:
            if dictionary_arguments["path"][-1] == "/":
                dictionary_arguments["path"] = (
                    f"{dictionary_arguments['path']}, {dictionary_arguments['file']}"
                )
            else:
                dictionary_arguments["path"] = (
                    f"{dictionary_arguments['path']}/{dictionary_arguments['file']}"
                )

        self.load_args_from_dictionary(dictionary_arguments)


class LsCommand(CommandBase):
    cmd = "ls"
    needs_admin = False
    help_cmd = "ls [directory]"
    description = "List directory."
    version = 2
    supported_ui_features = ["file_browser:list"]
    author = "@M_alphaaa"
    argument_class = LsArguments
    attackmapping = ["T1106", "T1083"]
    browser_script = BrowserScript(script_name="ls", author="@M_alphaaa", for_new_ui=True)
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, taskData: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        if host := taskData.args.get_arg("host"):
            if (
                taskData.Callback.OS.lower().startswith("linux")
                and taskData.Callback.Host != host.upper()
            ):
                raise Exception(
                    "Cannot get directory listings of remote hosts using ls on Linux. "
                    "Use the `ssh` command for listing remote files."
                )
        else:
            taskData.args.add_arg("host", taskData.Callback.Host)

        if not taskData.args.has_arg("file"):
            taskData.args.add_arg("file", "")

        path = taskData.args.get_arg("path")
        resp = PTTaskCreateTaskingMessageResponse(TaskID=taskData.Task.ID)
        if (host := taskData.args.get_arg("host")) and host != taskData.Callback.Host:
            resp.DisplayParams = f"\\\\{host}\\{path}"
        else:
            resp.DisplayParams = path
        return resp
