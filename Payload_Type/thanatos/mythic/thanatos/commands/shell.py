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
from mythic_container.MythicGoRPC import (
    SendMythicRPCArtifactCreate,
    MythicRPCArtifactCreateMessage,
)

# TODO: Refactor implementation


class ShellArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="command",
                type=ParameterType.String,
                description="Command to run.",
                display_name="Command to run.",
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_json_string(self.command_line)
            else:
                self.set_arg("command", self.command_line)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)


class ShellCommand(CommandBase):
    cmd = "shell"
    needs_admin = False
    help_cmd = "shell [command]"
    description = (
        "Execute a shell command with '/bin/bash -c' on Linux "
        "or 'cmd.exe /c' on Windows in a new thread"
    )
    version = 2
    author = "@M_alphaaa"
    argument_class = ShellArguments
    attackmapping = ["T1059"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )

    async def create_go_tasking(
        self, task_data: PTTaskMessageAllData
    ) -> PTTaskCreateTaskingMessageResponse:
        if task_data.Callback.Host == "Linux":
            shell = "/bin/bash -c "
        else:
            shell = "cmd.exe /c "

        await SendMythicRPCArtifactCreate(
            MythicRPCArtifactCreateMessage(
                TaskID=task_data.Task.ID,
                ArtifactMessage=shell + task_data.args.get_arg("command"),
                BaseArtifactType="Process Create",
            )
        )

        return PTTaskCreateTaskingMessageResponse(
            TaskID=task_data.Task.ID,
            Success=True,
            DisplayParams=task_data.args.get_arg("command"),
        )
