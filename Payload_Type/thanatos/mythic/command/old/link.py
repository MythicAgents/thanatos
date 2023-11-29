from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    SupportedOS,
    MythicTask,
    AgentResponse,
    CommandParameter,
    ParameterType,
    MythicStatus,
)
from mythic_container.MythicRPC import MythicRPC


class LinkArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="connection_info",
                cli_name="connection",
                type=ParameterType.ConnectionInfo,
            )
        ]

    async def parse_arguments(self):
        if len(self.command_line) == 0:
            raise Exception(
                "Link command requires arguments, but got empty command line."
            )
        if self.command_line[0] != "{":
            raise Exception("Require JSON blob of arguments, but got raw command line.")
        self.load_args_from_json_string(self.command_line)


class LinkCommand(CommandBase):
    cmd = "link"
    needs_admin = False
    help_cmd = "link [popup]"
    description = "Link to a new agent on a remote host or re-link back to a specified callback that's been unlinked via the `unlink` commmand."
    version = 1
    author = "@M_alphaaa"
    argument_class = LinkArguments
    attackmapping = ["T1570", "T1572", "T1021"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Windows, SupportedOS.Linux], builtin=True
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        link_host = task.args.get_arg("connection_info")["host"]
        link_profile = task.args.get_arg("connection_info")["c2_profile"]["name"]
        link_profile = (
            link_profile[len("thanatos_") :]
            if "thanatos_" in link_profile
            else link_profile
        )

        if "tcp" in link_profile:
            port = int(
                task.args.get_arg("connection_info")["c2_profile"]["parameters"]["port"]
            )

            if port <= 0 or port >= 65536:
                raise Exception("Port number for TCP connection is invalid")

            task.display_params = "{} => {}:{}".format(
                link_profile,
                task.args.get_arg("connection_info")["host"],
                port,
            )
        else:
            pipe_name = task.args.get_arg("connection_info")["c2_profile"][
                "parameters"
            ]["pipename"]

            task.display_params = "{} => {}:{}".format(
                link_profile,
                task.args.get_arg("connection_info")["host"],
                pipe_name,
            )

        return task

    async def process_response(self, response: AgentResponse):
        link_profile = response.task.args.get_arg("connection_info")["c2_profile"][
            "name"
        ]
        link_profile = (
            link_profile[len("thanatos_") :]
            if "thanatos_" in link_profile
            else link_profile
        )

        if "tcp" in link_profile:
            output = "Linked to {}:{}".format(
                response.task.args.get_arg("connection_info")["host"],
                response.task.args.get_arg("connection_info")["c2_profile"][
                    "parameters"
                ]["port"],
            )

        resp = await MythicRPC().execute(
            "create_output",
            task_id=response.task.id,
            output=output.encode(),
        )

        if resp.status != MythicStatus.Success:
            raise Exception(resp.error)
