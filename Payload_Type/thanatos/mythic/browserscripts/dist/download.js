"use strict";
function download(task, responses) {
    if (task.status.includes("error")) {
        var combined = responses.reduce(function (prev, cur) {
            return prev + cur;
        }, "");
        return { 'plaintext': combined };
    }
    else if (task.completed) {
        if (responses.length > 0) {
            try {
                return {
                    "download": [{
                            "agent_file_id": responses[0],
                            "variant": "contained",
                            "name": "Download " + task["display_params"]
                        }]
                };
            }
            catch (error) {
                console.log(error);
                var combined = responses.reduce(function (prev, cur) {
                    return prev + cur;
                }, "");
                return { 'plaintext': combined };
            }
        }
        else {
            return { "plaintext": "No data to display..." };
        }
    }
    else if (task.status === "processed") {
        if (responses.length > 0) {
            var task_data = JSON.parse(responses[0]);
            console.log(task_data);
            return { "plaintext": "Downloading a file with " + task_data["total_chunks"] + " total chunks..." };
        }
        return { "plaintext": "No data yet..." };
    }
    else {
        // this means we shouldn't have any output
        return { "plaintext": "Not response yet from agent..." };
    }
}
