from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
    SupportedOS,
    MythicTask,
    MythicStatus,
    PTTaskMessageAllData,
    PTTaskProcessResponseMessageResponse,
)


class SetEnvArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="name",
                type=ParameterType.String,
                description="Environment variable name.",
                display_name="Environment variable name.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
            CommandParameter(
                name="value",
                type=ParameterType.String,
                description="Environment variable value.",
                display_name="Environment variable value.",
                parameter_group_info=[ParameterGroupInfo(ui_position=2)],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                self.verify_required_args_have_values()
                args = self.command_line.split(" ")
                self.set_arg("name", args[0])
                self.set_arg("value", args[1])

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class SetEnvCommand(CommandBase):
    cmd = "setenv"
    needs_admin = False
    help_cmd = "setenv [name] [value]"
    description = "Sets an environment variable."
    version = 1
    author = "@M_alphaaa"
    argument_class = SetEnvArguments
    attackmapping = []
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        name = task.args.get_arg("name")
        value = task.args.get_arg("value")

        task.display_params = f"{name} {value}"
        if "=" in name or "=" in value:
            task.set_stderr("Cannot have '=' in environment variables.")
            task.status = MythicStatus.Error

        return task

    async def process_response(
        self, task: PTTaskMessageAllData, response: str
    ) -> PTTaskProcessResponseMessageResponse:
        pass
