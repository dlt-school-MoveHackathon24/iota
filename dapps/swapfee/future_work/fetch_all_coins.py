import subprocess

def execute_command(command):
    """Executes a shell command and returns the output."""
    try:
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, shell=True)
        return result.stdout, result.stderr
    except Exception as e:
        print(f"Error during command execution: {e}")
        return None, str(e)

def fetch_all_coins(address):
    """It executes the command to obtain all objects and saves the output in a file."""
    command_objects = f"iota client objects {address}"
    output, error = execute_command(command_objects)

    if error:
        print(f"Errore: {error}")
        return

    # Salva l'output in un file
    with open("tutti_coin.txt", "w") as file:
        file.write(output)

    print("Output saved in tutti_coin.txt.")

if __name__ == "__main__":
    print("This script is intended to be imported, not executed directly.")
