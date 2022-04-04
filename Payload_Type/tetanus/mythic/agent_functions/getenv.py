from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    MythicTask,
    AgentResponse,
    BrowserScript,
    CommandAttributes,
    SupportedOS
)


class GetEnvArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = []

    async def parse_arguments(self):
        pass


class GetEnvCommand(CommandBase):
    cmd = "getenv"
    needs_admin = False
    help_cmd = "getenv"
    description = "Get all environment variables."
    version = 1
    author = "@M_alphaaa"
    supported_ui_features = ["callback_table:getenv"]
    argument_class = GetEnvArguments
    attackmapping = ["T1082"]
    browser_script = BrowserScript(
        script_name="getenv", author="@M_alphaaa", for_new_ui=True
    )
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Windows, SupportedOS.Linux ],
        builtin=True,
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        return task

    async def process_response(self, response: AgentResponse):
        pass
