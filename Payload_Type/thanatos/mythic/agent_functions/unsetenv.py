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


class UnsetEnvArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="variable",
                type=ParameterType.String,
                cli_name="var",
                description="Variable to unset.",
                display_name="Variable to unset.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                self.set_arg("variable", self.command_line)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class UnsetEnvCommand(CommandBase):
    cmd = "unsetenv"
    needs_admin = False
    help_cmd = "unsetenv [variable]"
    description = "Unset an environment variable"
    version = 1
    author = "@M_alphaaa"
    supported_ui_features = ["unsetenv"]
    argument_class = UnsetEnvArguments
    attackmapping = []
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        task.display_params = str(task.args.get_arg("variable"))
        return task

    async def process_response(
        self, task: PTTaskMessageAllData, response: str
    ) -> PTTaskProcessResponseMessageResponse:
        pass
