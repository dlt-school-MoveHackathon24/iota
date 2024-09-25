import re

def read_whitelisted_coin_types(file_path):
    """Reads coin types from the whitelistedCoinTypes.ts file"""
    with open(file_path, 'r') as file:
        lines = file.readlines()
    
    #Extract entries between single inverted commas
    whitelisted_coins = []
    for line in lines:
        match = re.findall(r"'([^']*)'", line)
        if match:
            whitelisted_coins.extend(match)
    return whitelisted_coins


def read_exchange_rates(file_path):
    """Reads the exchangeRates object from the file calculateDynamicGasFee.ts"""
    with open(file_path, 'r') as file:
        content = file.read()
    
    #  Use regex to find all coin types defined in the exchangeRates object
    pattern = re.compile(r"'([^']*)':\s*\{ rateInIota:\s*[\d\.]+,\s*rateInToken:\s*[\d\.]+ \}")
    exchange_rates = pattern.findall(content)
    
    return exchange_rates


def add_missing_exchange_rates(whitelisted_coins, exchange_rates, calculate_file_path):
    """Adds missing coin types in the file calculateDynamicGasFee.ts"""
    missing_coins = [coin for coin in whitelisted_coins if coin not in exchange_rates]

    if not missing_coins:
        print("All coin types already have a match in the file calculateDynamicGasFee.ts.")
        return

    # Ask the user to enter rateInIota and rateInToken for each missing coin
    new_lines = []
    for coin in missing_coins:
        print(f"The coin {coin} is missing from calculateDynamicGasFee.ts.")
        rate_in_iota = input(f"Enter rateInIota: ")
        rate_in_token = input(f"Enter rateInToken for  {coin}: ")
        new_line = f"    '{coin}': {{ rateInIota: {rate_in_iota}, rateInToken: {rate_in_token} }},\n"
        new_lines.append(new_line)

    #Insert the new lines before the comment ‘// Add other tokens as needed’.
    with open(calculate_file_path, 'r') as file:
        lines = file.readlines()

    for index, line in enumerate(lines):
        if "// Add other tokens as needed" in line:
            #  Insert new lines before the comment
            lines = lines[:index] + new_lines + lines[index:]
            break

    # Overwrite the file with the newly added lines
    with open(calculate_file_path, 'w') as file:
        file.writelines(lines)

    print(f"The new lines have been added to {calculate_file_path}.")


if __name__ == "__main__":
    whitelisted_coins = read_whitelisted_coin_types("whitelistedCoinTypes.ts")
    exchange_rates = read_exchange_rates("calculateDynamicGasFee.ts")
    add_missing_exchange_rates(whitelisted_coins, exchange_rates, "calculateDynamicGasFee.ts")
