from mythic_payloadtype_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    MythicTask,
    AgentResponse,
    CommandParameter,
    ParameterType,
    ParameterGroupInfo,
    CommandAttributes,
    SupportedOS,
    MythicStatus
)
from mythic_payloadtype_container.MythicRPC import MythicRPC


class SocksArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="action",
                type=ParameterType.ChooseOne,
                choices=["start", "stop"],
                default_value="start",
                description="Start or Stop socks through this callback.",
                parameter_group_info=[
                    ParameterGroupInfo(
                        ui_position=1
                    )
                ]
            ),
            CommandParameter(
                name="port",
                type=ParameterType.Number,
                description="Port to open on Mythic for socks5 proxying",
                parameter_group_info=[
                    ParameterGroupInfo(
                        ui_position=2
                    )
                ]
            ),
        ]

    async def parse_arguments(self):
        self.load_args_from_json_string(self.command_line)


class SocksCommand(CommandBase):
    cmd = "socks"
    needs_admin = False
    help_cmd = "socks [action] [port number]"
    description = "Enable SOCKS 5 compliant proxy on the agent such that you may proxy data in from an outside machine into the target network."
    version = 1
    is_exit = False
    is_file_browse = False
    is_process_list = False
    is_download_file = False
    is_upload_file = False
    is_remove_file = False
    author = "@M_alphaaa"
    argument_class = SocksArguments
    attackmapping = ["T1572"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Windows, SupportedOS.Linux ],
        builtin=False,
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        if task.args.get_arg("action") == "start":
            resp = await MythicRPC().execute("control_socks", task_id=task.id, start=True, port=task.args.get_arg("port"))
            if resp.status != MythicStatus.Success:
                task.status = MythicStatus.Error
                raise Exception(resp.error)
        else:
            resp = await MythicRPC().execute("control_socks", task_id=task.id, stop=True, port=task.args.get_arg("port"))
            if resp.status != MythicStatus.Success:
                task.status = MythicStatus.Error
                raise Exception(resp.error)
        return task

    async def process_response(self, response: AgentResponse):
        pass
