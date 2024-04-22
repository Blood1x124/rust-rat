import { invoke } from "@tauri-apps/api/tauri";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { ShellCommandType } from "../../types";

export const getOutput = async (
  command: string,
  setCommand: React.Dispatch<React.SetStateAction<ShellCommandType[]>>,
  shellStatus: string,
  id: string
): Promise<JSX.Element | string> => {
  return new Promise((resolve) => {
    switch (command.toLowerCase()) {
      case "!help":
        resolve(
          <div>
            Available commands: <br />
            <span className="text-primary ml-3">!clear</span> - Clear the
            terminal
          </div>
        );
        break;
      case "!clear":
        setCommand([]);
        resolve("");
        break;
      default:
        if (shellStatus == "true") {
          let output: string = "";
          let timer: number | null = null;
          let unlisten: UnlistenFn | undefined;

          listen("client_shellout", (event) => {
            output += event.payload + "\n";
            console.log(event.payload);
            if (timer !== undefined && timer) clearTimeout(timer);
            timer = setTimeout(() => {
              resolve(<div style={{ whiteSpace: "pre-wrap" }}>{output}</div>);
              if (unlisten) unlisten();
            }, 250);
          }).then((unlistenFn) => {
            unlisten = unlistenFn;
          });

          invoke("execute_shell_command", { id, run: command }).then(() => {
            timer = setTimeout(() => {
              resolve(<div style={{ whiteSpace: "pre-wrap" }}>{output}</div>);
              if (unlisten) unlisten();
            }, 250);
          });
        } else
          resolve(
            <div>
              <span className="text-error">Shell is not started.</span>
            </div>
          );
    }
  });
};
