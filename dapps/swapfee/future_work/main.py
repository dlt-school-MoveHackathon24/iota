import subprocess
from fetch_all_coins import fetch_all_coins
from extract_types import extract_types_from_file
from fetch_coin_types import extract_object_ids, fetch_types_for_objects
from update_whitelisted import update_whitelisted_coins
from black_list import clean_blacklisted_coins

def main():
    # Ask the user to enter an address
    address = input("Please enter the address: ")

    # Run the various scripts in sequence
    fetch_all_coins(address)  # Pass the address to fetch_all_coins

    # Step 1: Extract objects
    object_ids = extract_object_ids("tutti_coin.txt")
    
    # Step 2: Retrieve the types
    fetch_types_for_objects(object_ids)
    
    # Extract the types from the file ‘types_output.txt’ and save them in ‘tutti_i_type_coin.txt’.
    extract_types_from_file("types_output.txt", "tutti_i_type_coin.txt")

    # Step 3: Clean the types from the blacklist before updating the TypeScript file
    clean_blacklisted_coins("tutti_i_type_coin.txt", "black_list.txt")

    # Step 4: Update the TypeScript file with the new filtered types
    update_whitelisted_coins("tutti_i_type_coin.txt", "whitelistedCoinTypes.ts")

    # Step 5: Run the update_rate_coins.py script
    try:
        subprocess.run(["python3", "update_rate_coins.py"], check=True)
        print("update_rate_coins.py executed successfully.")
    except subprocess.CalledProcessError as e:
        print(f"Error executing update_rate_coins.py: {e}")

if __name__ == "__main__":
    main()
