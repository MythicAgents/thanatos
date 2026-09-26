from mythic_container.MythicCommandBase import (
    BrowserScript,
    CommandAttributes,
    CommandBase,
    SupportedOS,
    TaskArguments,
)


class NetstatArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = []

    async def parse_arguments(self):
        pass


class NetstatCommand(CommandBase):
    cmd = "netstat"
    needs_admin = False
    help_cmd = "netstat"
    description = "Get all active network connections & sockets"
    version = 1
    author = "@maclarel"
    argument_class = NetstatArguments
    attackmapping = ["T1049"]
    browser_script = BrowserScript(
        script_name="netstat", author="@M_alphaaa", for_new_ui=True
    )
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
    )
