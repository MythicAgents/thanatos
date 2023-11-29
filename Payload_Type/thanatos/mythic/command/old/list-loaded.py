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
    SendMythicRPCCommandSearch,
    MythicRPCCommandSearchMessage,
    MythicRPCTaskUpdateMessage,
    SendMythicRPCCallbackAddCommand,
    MythicRPCCallbackAddCommandMessage,
    SendMythicRPCResponseCreate,
    MythicRPCResponseCreateMessage,
)
from mythic_container.MythicRPC import MythicRPC


class ListLoadedArguments(TaskArguments):
    def __init__(self, command_line, **kwargs):
        super().__init__(command_line, **kwargs)
        self.args = []

    async def parse_arguments(self):
        pass


class ListLoadedCommand(CommandBase):
    cmd = "list-loaded"
    needs_admin = False
    help_cmd = "list-loaded"
    description = "List commands currently loaded into the agent."
    version = 1
    author = "@M_alphaaa"
    argument_class = ListLoadedArguments
    script_only = True
    attackmapping = ["T1083"]
    attributes = CommandAttributes(
        supported_os=[SupportedOS.Linux, SupportedOS.Windows],
        builtin=True,
    )

    async def create_go_tasking(self, task: PTTaskMessageAllData) -> MythicTask:
        raise Exception("{}".format(json.dumps(task.callback.__dict__, indent=2)))

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
            "get_commands",
            callback_id=task.callback.id,
            loaded_only=True,
        )

        if loaded_commands.status != MythicStatus.Success:
            raise Exception(json.dumps(loaded_commands.__dict__, indent=2))

        loaded_command_names = set(
            [command["cmd"] for command in loaded_commands.response]
        )

        commands = sorted(loaded_command_names.intersection(unloadable_command_names))

        output = ""
        for i, command in enumerate(commands):
            output += f"{i + 1}. {command}\n"

        if output == "":
            output = "No commands have been loaded into the agent."

        await SendMythicRPCResponseCreate(
            MythicRPCResponseCreateMessage(TaskID=task.id, Response=output.encode())
        )

        task.completed = True
        task.status = MythicStatus.Success
        return task
