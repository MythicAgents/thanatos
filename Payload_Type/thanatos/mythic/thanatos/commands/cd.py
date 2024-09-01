from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
    SupportedOS,
    PTTaskMessageAllData,
    PTTaskCreateTaskingMessageResponse,
)


class CdArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="directory",
                type=ParameterType.String,
                description="Directory to cd into.",
                display_name="Directory to cd into.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            self.set_arg("directory", self.get_command_line())

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class CdCommand(CommandBase):
    cmd = "cd"
    needs_admin = False
    help_cmd = "cd [directory]"
    description = "Change working directory (can be relative)."
    version = 2
    author = "@M_alphaaa"
    argument_class = CdArguments
    attackmapping = ["T1083"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, task_data: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        task_data.display_params = str(task_data.args.get_arg("directory"))
        return PTTaskCreateTaskingMessageResponse(
            TaskID=task_data.Task.ID,
            DisplayParams=str(task_data.args.get_arg("directory")),
            Success=True,
        )
