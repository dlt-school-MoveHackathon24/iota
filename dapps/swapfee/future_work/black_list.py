def clean_blacklisted_coins(types_file, blacklist_file):
    """It removes any line in the types_file that is also present in the blacklist_file."""
    
    # Read the file containing the coin types
    with open(types_file, "r") as file:
        coin_types = file.readlines()

    # Read the blacklist file
    with open(blacklist_file, "r") as file:
        blacklisted_coins = {line.strip() for line in file}

    # Filter coin types by removing those on the blacklist
    filtered_coin_types = [coin_type for coin_type in coin_types if coin_type.strip() not in blacklisted_coins]

    # Overwrite the types_file with the filtered list
    with open(types_file, "w") as file:
        file.writelines(filtered_coin_types)

    print(f"Blacklist successfully applied to {types_file}.")

if __name__ == "__main__":
    clean_blacklisted_coins("tutti_i_types_coin.txt", "black_list.txt")
    update_whitelisted_coins("tutti_i_types_coin.txt", "whitelistedCoinTypes.ts")
