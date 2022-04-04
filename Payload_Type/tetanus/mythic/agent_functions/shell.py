from mythic_payloadtype_container.MythicRPC import MythicRPC
from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    MythicTask,
    AgentResponse,
    CommandParameter,
    ParameterType,
    CommandAttributes,
    SupportedOS
)


class ShellArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="command",
                type=ParameterType.String,
                description="Command to run.",
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
    description = "Execute a shell command with 'bash -c' on linux or '/cmd.exe /c' on windows in a new thread"
    version = 1
    author = "@M_alphaaa"
    argument_class = ShellArguments
    attackmapping = ["T1059"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Windows, SupportedOS.Linux ],
        builtin=True,
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        await MythicRPC().execute(
            "create_artifact",
            task_id=task.id,
            artifact="{}".format(task.args.command_line),
            artifact_type="Process Create",
        )

        task.display_params = task.args.get_arg("command")
        return task

    async def process_response(self, response: AgentResponse):
        pass
