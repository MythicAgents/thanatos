import json

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


class RmArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="path",
                type=ParameterType.String,
                description="File or directory to remove.",
                display_name="File or directory to remove.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
        ]

    async def parse_arguments(self):
        try:
            tmp_json = json.loads(self.command_line)
            if "file" in tmp_json.keys() and "host" in tmp_json.keys():
                self.add_arg("host", tmp_json["host"])
                self.set_arg("path", f"{tmp_json['path']}/{tmp_json['file']}")
            else:
                self.set_arg("path", tmp_json["path"])
        except Exception:
            self.set_arg("path", self.command_line)

    async def parse_dictionary(self, dictionary_arguments):
        if "file" in dictionary_arguments:
            dictionary_arguments["path"] = (
                f"{dictionary_arguments['path']}/{dictionary_arguments['file']}"
            )

        self.load_args_from_dictionary(dictionary_arguments)


class RmCommand(CommandBase):
    cmd = "rm"
    needs_admin = False
    help_cmd = "rm [path]"
    description = "Delete a file or directory."
    version = 1
    supported_ui_features = ["file_browser:remove"]
    author = "@M_alphaaa"
    argument_class = RmArguments
    attackmapping = ["T1070.004", "T1565"]
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

        path = taskData.args.get_arg("path")
        resp = PTTaskCreateTaskingMessageResponse(TaskID=taskData.Task.ID)
        if (host := taskData.args.get_arg("host")) and host != taskData.Callback.Host:
            resp.DisplayParams = f"\\\\{host}\\{path}"
        return resp
