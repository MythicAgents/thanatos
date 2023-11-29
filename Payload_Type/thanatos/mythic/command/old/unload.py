import asyncio
import base64
import tempfile
import traceback
import hcl
import os
import json
from pathlib import Path
from shutil import copytree
from mythic_container.MythicCommandBase import (
    TaskArguments,
    CommandBase,
    CommandAttributes,
    CommandParameter,
    ParameterType,
    SupportedOS,
    ParameterGroupInfo,
    MythicStatus,
    MythicTask,
    PTTaskCompletionFunctionMessage,
    PTTaskCompletionFunctionMessageResponse,
    PTRPCDynamicQueryFunctionMessage,
    PTRPCDynamicQueryFunctionMessageResponse,
)
from mythic_container.MythicGoRPC import (
    SendMythicRPCTaskUpdate,
    MythicRPCTaskUpdateMessage,
    SendMythicRPCCommandSearch,
    MythicRPCCommandSearchMessage,
    SendMythicRPCCallbackRemoveCommand,
    MythicRPCCallbackRemoveCommandMessage,
    SendMythicRPCResponseCreate,
    MythicRPCResponseCreateMessage,
)
from mythic_container.MythicRPC import MythicRPC


class UnloadArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = [
            CommandParameter(
                name="command",
                type=ParameterType.ChooseOne,
                display_name="Command to unload.",
                description="Command to unload.",
                dynamic_query_function=self.get_unloadable_commands,
                parameter_group_info=[ParameterGroupInfo(ui_position=1)],
            ),
        ]

    async def parse_arguments(self):
        if self.tasking_location == "command_line":
            if self.command_line[0] == "{":
                self.load_args_from_dictionary(self.command_line)
            else:
                self.set_arg("command", self.command_line)

    async def parse_dictionary(self, dictionary_arguments):
        self.load_args_from_dictionary(dictionary_arguments)

    async def get_unloadable_commands(
        self, msg: PTRPCDynamicQueryFunctionMessage
    ) -> PTRPCDynamicQueryFunctionMessageResponse:
        all_commands = await SendMythicRPCCommandSearch(
            MythicRPCCommandSearchMessage(SearchPayloadTypeName="thanatos")
        )

        unloadable_commands = set(
            filter(
                lambda c: c.Attributes["builtin"] is False,
                list(all_commands.Commands),
            )
        )
        unloadable_command_names = [command.Name for command in unloadable_commands]

        loaded_commands = await MythicRPC().execute(
            "get_commands", callback_id=msg.Callback, loaded_only=True
        )
        loaded_command_names = set(
            [command["cmd"] for command in loaded_commands.response]
        )

        return PTRPCDynamicQueryFunctionMessageResponse(
            Success=True,
            Choices=sorted(loaded_command_names.intersection(unloadable_command_names)),
        )


async def remove_callback_command(
    task: PTTaskCompletionFunctionMessage,
) -> PTTaskCompletionFunctionMessageResponse:
    params = json.loads(task.TaskData.Task.Params)
    command_name = params["command"]

    await SendMythicRPCCallbackRemoveCommand(
        MythicRPCCallbackRemoveCommandMessage(
            TaskID=task.TaskData.Task.ID, Commands=[command_name]
        )
    )

    await SendMythicRPCResponseCreate(
        MythicRPCResponseCreateMessage(
            TaskID=task.TaskData.Task.ID,
            Response=f"Unloaded {command_name} from the agent.".encode(),
        )
    )

    return PTTaskCompletionFunctionMessageResponse(
        TaskID=task.TaskData.Task.ID,
        ParentTaskId=0,
        Completed=True,
        Success=True,
    )


class UnloadCommand(CommandBase):
    cmd = "unload"
    needs_admin = False
    help_cmd = "unload [command name]"
    description = "Unload a command from the agent."
    version = 1
    author = "@M_alphaaa"
    argument_class = UnloadArguments
    attackmapping = ["T1083"]
    completion_functions = {"add_callback_command": remove_callback_command}
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
        builtin=True,
    )

    async def create_tasking(self, task: MythicTask) -> MythicTask:
        command = task.args.get_arg("command")
        task.display_params = command
        task.completed_callback_function = "add_callback_command"
        return task
