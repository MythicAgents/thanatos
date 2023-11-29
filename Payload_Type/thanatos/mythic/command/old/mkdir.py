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


class MkdirArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="directory",
                type=ParameterType.String,
                cli_name="dir",
                description="Directory to make.",
                display_name="Directory to make.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            )
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                self.verify_required_args_have_values()
                args = self.command_line
                self.set_arg("directory", args)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class MkdirCommand(CommandBase):
    cmd = "mkdir"
    needs_admin = False
    help_cmd = "mkdir [directory]"
    description = "Make a new directory."
    version = 1
    author = "@M_alphaaa"
    argument_class = MkdirArguments
    attackmapping = ["T1106"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        directory = task.args.get_arg("directory")
        task.display_params = directory
        return task

    async def process_response(
        self, task: PTTaskMessageAllData, response: str
    ) -> PTTaskProcessResponseMessageResponse:
        pass
