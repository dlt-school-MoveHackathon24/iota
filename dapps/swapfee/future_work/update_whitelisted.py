def update_whitelisted_coins(input_file, ts_file):
    """It reads the types from the input file and overwrites those in the TypeScript file."""
   
    # Read types from the input file
    with open(input_file, "r") as file:
        coin_types = file.readlines()

    # Opens the TypeScript file in read mode and creates a list for the lines
    with open(ts_file, "r") as file:
        ts_lines = file.readlines()

    # Find the index of the start and end line
    start_index = -1
    end_index = -1

    for index, line in enumerate(ts_lines):
        if "export const whitelistedCoinTypes: string[] =" in line:
            start_index = index
        if "// Add other coin types as needed" in line:
            end_index = index
            break

    if start_index == -1 or end_index == -1:
        print(f"Start or end line not found in {ts_file}.")
        return

    # Update the lines between start_index and end_index
    new_types = [f"  {coin_type.strip()},\n" for coin_type in coin_types if coin_type.strip()]
    
    # Create the new list of rows
    ts_lines = ts_lines[:start_index + 1] + new_types + ts_lines[end_index:]

    # Writes the updated lines to the TypeScript file
    with open(ts_file, "w") as file:
        file.writelines(ts_lines)

    print(f"Types successfully updated in {ts_file}.")

if __name__ == "__main__":
    update_whitelisted_coins("tutti_i_type_coin.txt", "whitelistedCoinTypes.ts")
