function(task, responses){
  if (task.status.includes("error")) {
    return {
      "plaintext": responses.reduce((prev, cur) => prev + cur, ""),
    };
  }

  if (responses.length <= 0) {
    return {"plaintext": "No response yet from agent..."}
  }

  const title = "Network Connections";
  const headers = [
      {"plaintext": "protocol", "type": "string", "cellStyle": {}, "width": 125},
      {"plaintext": "local address", "type": "string", "cellStyle": {}, "width": 300},
      {"plaintext": "local port", "type": "number", "cellStyle": {}, "width": 200},
      {"plaintext": "remote address", "type": "string", "cellStyle": {}, "width": 300},
      {"plaintext": "remote port", "type": "number", "cellStyle": {}, "width": 200},
      {"plaintext": "state", "type": "string", "cellStyle": {}, "width": 200},
      {"plaintext": "pids", "type": "string", "cellStyle": {}, "width": 125},
  ];

  let rows = [];

  for (const response of responses) {
    let data = {};
    try {
      data = JSON.parse(response);
    } catch (error) {
      console.log(error);
      return {
        "plaintext": responses.reduce((prev, cur) => prev + cur, ""),
      }
    }

    rows.push.apply(rows, data.map((netstatEntry) => {
      return {
        "protocol": {
          "plaintext": netstatEntry["proto"],
        },
        "local address": {
          "plaintext": netstatEntry["local_addr"],
          "copyIcon": netstatEntry["local_addr"] !== null && netstatEntry["local_addr"].trim().length !== 0,
        },
        "local port": {
          "plaintext": netstatEntry["local_port"],
          "copyIcon": true,
        },
        "remote address": {
          "plaintext": netstatEntry["remote_addr"],
          "copyIcon": netstatEntry["remote_addr"] !== null && netstatEntry["remote_addr"].trim().length !== 0,
        },
        "remote port": {
          "plaintext": netstatEntry["remote_port"],
          "copyIcon": netstatEntry["remote_port"] !== null && netstatEntry["remote_port"] !== 0,
        },
        "state": {
          "plaintext": netstatEntry["state"],
        },
        "pids": {
          "plaintext": netstatEntry["associated_pids"]?.join(),
          "copyIcon": netstatEntry["associated_pids"] !== null && netstatEntry["associated_pids"].length !== 0,
        },
      }
    }));
  }

  if (rows.length == 0) {
    return {
      "plaintext": responses.reduce((prev, cur) => prev + cur, ""),
    }
  } else {
    return {
      "table": [
        {
          "title": title,
          "headers": headers,
          "rows": rows,
        }
      ]
    };
  }
}
