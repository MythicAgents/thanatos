from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
    SupportedOS,
    MythicTask,
)


class CatArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="file",
                type=ParameterType.String,
                description="File to display the contents of.",
                display_name="File to display the contents of.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
        ]

    async def parse_arguments(self):
        if (
            self.tasking_location == "command_line"
            or self.tasking_location == "browserscript"
        ):
            if self.command_line[0] == "{":
                self.load_args_from_dictionary(self.command_line)
            else:
                self.set_arg("file", self.get_command_line())

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class CatCommand(CommandBase):
    cmd = "cat"
    needs_admin = False
    help_cmd = "cat [file path]"
    description = "Display the contents of a file."
    version = 1
    author = "@M_alphaaa"
    supported_ui_features = ["cat"]
    argument_class = CatArguments
    attackmapping = ["T1005", "T1039", "T1025"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        task.display_params = str(task.args.get_arg("file"))
        return task
