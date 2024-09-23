import subprocess

deployed_package = "0x6e6bedea2f03086752e957d8ce1a7afde6a3b10d512efc68613953d14e88f6cc"
assetCoin = "0x9a80f8d5de67c65b0dbe4e38cfff9624f35ee9705a2def73fd0d6a7e156ea161"
base_coin = "0x088f348f5c8e40a68dad72952c6042dfe8c1ec6fc23b6679cc69fabac568534f"


def run_command(command):
    """Run a shell command and print the output."""
    print(f"Executing: {command}")
    result = subprocess.run(command, shell=True, capture_output=True, text=True)
    if result.returncode == 0:
        print("Command executed successfully")
        print(result.stdout)
    else:
        print("Error executing command")
        print(result.stderr)


def deploy():
    """Deploy a package."""
    run_command("iota client publish --gas-budget 500000000")


def mint_examplecoin():
    """Mint ExampleCoin for a specific address."""
    package = input(f"Enter package address (default: {assetCoin}): ") or assetCoin
    mint_object = input("Enter the mint object address (default: 0x5fa966827fe9181a65a14e110b15097c02693b25fcd475ed19d053ed706cabcd): ") or "0x5fa966827fe9181a65a14e110b15097c02693b25fcd475ed19d053ed706cabcd"
    amount = input("Enter the amount to mint (default: 1000): ") or "1000"
    gas_budget = input("Enter the gas budget (default: 5000000): ") or "5000000"

    command = f"iota client call --package {package} --module ExampleCoin --function mint " \
              f"--type-args {package}::ExampleCoin::EXAMPLECOIN --args {mint_object} {amount} " \
              f"--gas-budget {gas_budget}"
    run_command(command)


def create_strategy_not_for_coin():
    """Create a strategy object (non-coin-based)."""
    package = input(f"Enter package address (default: {deployed_package}): ") or deployed_package
    id_strategy = input("Enter the strategy type (1 for equal distribution, 2 for dynamic distribution): ") or "2"
    amount_each = input("Enter the amount for each user (default: 10): ") or "10"
    start_time = input("Enter the start timestamp (default: 1726743803978): ") or "1726743803978"
    duration = input("Enter the duration (in milliseconds) (default: 1000000): ") or "1000000"
    type_v = input("Enter type for vester (default: u64): ") or "u64"
    
    
    gas_budget = "50000000"

    command = f"iota client ptb " \
              f"--move-call {package}::vesting::create_strategy_not_for_coin  {id_strategy} {amount_each}"\
              f" --assign strategy --move-call {package}::vesting::create_vester_and_share " \
              f" \"<{assetCoin}::ExampleCoin::EXAMPLECOIN, {type_v}>\" " \
              f" strategy {start_time} {duration} --gas-budget {gas_budget}"
    run_command(command)


def create_strategy_for_coin():
    """Create a strategy object for a specific coin."""
    package = input(f"Enter package address (default: {deployed_package}): ") or deployed_package
    id_strategy = "3"
    amount_each = input("Enter the amount for each user (default: 10): ") or "10"
    coin_type = input(f"Enter the coin type (default: {base_coin}::TestCoin::TESTCOIN): ") or f"{base_coin}::TestCoin::TESTCOIN"
    gas_budget = "50000000"

    command = f"iota client call --package {package} --module vesting --function create_strategy_for_coin " \
              f" --type-args {coin_type} --args {id_strategy} {amount_each}" \
              f" --gas-budget {gas_budget}"
    run_command(command)


def create_vester(strategy_id):
    """Create a new vester for the vesting contract."""
    package = input(f"Enter package address (default: {deployed_package}): ") or deployed_package
    start_time = input("Enter the start timestamp (default: 1726743803978): ") or "1726743803978"
    duration = input("Enter the duration (in milliseconds) (default: 1000000): ") or "1000000"
    type_v = input("Enter type for vester (default: u64): ") or "u64"
    strategy_object = input(f"Enter strategy object address (default: {strategy_id}): ") or strategy_id
    gas_budget = "50000000"

    command = f"iota client ptb --assign pack @{package} --assign coinA @{assetCoin} "\
              f" --move-call pack::vesting::create_vester " \
              f" \"<{assetCoin}::ExampleCoin::EXAMPLECOIN, {type_v}>\" " \
              f" {start_time} {duration} @{strategy_object} --gas-budget {gas_budget}"
    run_command(command)


def transfer_object(vester_id, coin_id):
    """Transfer an object to a specified address."""
    vester = input(f"Enter the vester object ID (default: {vester_id}): ") or vester_id
    object_id = input(f"Enter the coin ID (default: {coin_id}): ") or coin_id
    gas_budget = "50000000"

    command = f"iota client transfer --to {vester} --object-id {object_id} --gas-budget {gas_budget}"
    run_command(command)


def initialize_vester(vester_id, coin_id):
    """Initialize a vesting contract with the supplied funds."""
    package = input(f"Enter package address (default: {deployed_package}): ") or deployed_package
    vester = input(f"Enter the vester object ID (default: {vester_id}): ") or vester_id
    object_id = input(f"Enter the coin ID (default: {coin_id}): ") or coin_id
    type_v = input("Enter type for vester (default: u64): ") or "u64"
    gas_budget = "50000000"

    command = f"iota client ptb --assign pack @{package} --assign coinA @{assetCoin} "\
              f" --move-call pack::vesting::initialize_vester " \
              f" \"<{assetCoin}::ExampleCoin::EXAMPLECOIN, {type_v}>\" " \
              f"  @{vester} @{object_id} --gas-budget {gas_budget}"
    run_command(command)


def release_vested_tokens(vester_id):
    """Release vested tokens to a user."""
    package = input(f"Enter package address (default: {deployed_package}): ") or deployed_package
    vester = input(f"Enter the vester object ID (default: {vester_id}): ") or vester_id
    type_v = input("Enter type for vester (default: u64): ") or "u64"
    gas_budget = "50000000"

    command = f"iota client call --package {package} " \
              f"--module vesting --function release_coins --type-args {package}::ExampleCoin::EXAMPLECOIN " \
              f"--args {vester_id} 0x0000000000000000000000000000000000000000000000000000000000000006 " \
              f"--gas-budget {gas_budget}"
    run_command(command)


def release_vested_tokens_by_coin(vester_id, coin_base):
    """Release vested tokens to a user by a coin-based strategy."""
    package = input(f"Enter package address (default: {deployed_package}): ") or deployed_package
    vester = input(f"Enter the vester object ID (default: {vester_id}): ") or vester_id
    object_id = input(f"Enter the coin ID (default: {coin_base}): ") or coin_base
    gas_budget = "50000000"

    command = f"iota client ptb --assign pack @{package} "\
              f"--move-call pack::vesting::release_coins_by_coinbase "\
              f" \"<{assetCoin}::ExampleCoin::EXAMPLECOIN, {base_coin}::TestCoin::TESTCOIN>\" " \
              f"@{vester_id} 0x0000000000000000000000000000000000000000000000000000000000000006 "\
              f"{object_id} --gas-budget {gas_budget}"
    run_command(command)


def collect_not_vested_coins(vester_id):
    """Collect any coins that were not vested."""
    package = input(f"Enter package address (default: {deployed_package}): ") or deployed_package
    vester = input(f"Enter the vester object ID (default: {vester_id}): ") or vester_id
    type_v = input("Enter type for vester (default: u64): ") or "u64"
    gas_budget = "50000000"

    command = f"iota client ptb --assign pack @{package} --assign coinA @{assetCoin} "\
              f"--move-call pack::vesting::collect_not_vested_coins " \
              f"\"<{assetCoin}::ExampleCoin::EXAMPLECOIN, {type_v}>\" @{vester} " \
              f"@0x0000000000000000000000000000000000000000000000000000000000000006 --gas-budget {gas_budget}"
    run_command(command)


def call_other_function():
    """Call any other arbitrary function on the vesting contract."""
    package = input(f"Enter package address (default: {deployed_package}): ") or deployed_package
    type_args = input("Enter the type-args: ")
    f_name = input("Enter the function name: ")
    args = input("Enter the arguments (separated by spaces): ")
    gas_budget = input("Enter the gas budget (default: 50000000): ") or "50000000"

    command = f"iota client ptb --assign pack @{package} --assign coinA @{assetCoin} "\
              f"--move-call pack::vesting::{f_name} " \
              f"\"{type_args}\" {args} --gas-budget {gas_budget}"
    run_command(command)


def main():
    global deployed_package
    vester_id = coin_id = strategy_id = ""
    actions_completed = {
        "0": False,
        "1": False,
        "2": False,
        "3": False,
        "4": False,
        "5": False,
        "6": False,
        "7": False,
        "77": False,
        "8": False,
        "9": False,
    }

    while True:
        print("\nChoose an action:")
        print(f"0. Deploy package {'✔' if actions_completed['0'] else ''}")
        print(f"1. Mint ExampleCoin {'✔' if actions_completed['1'] else ''}")
        print(f"2. Create Strategy Object (Not for Coin) {'✔' if actions_completed['2'] else ''}")
        print(f"3. Create Strategy Object for Coin {'✔' if actions_completed['3'] else ''}")
        print(f"4. Create Vester {'✔' if actions_completed['4'] else ''}")
        print(f"5. Transfer Coin To the Vester {'✔' if actions_completed['5'] else ''}")
        print(f"6. Initialize Vester {'✔' if actions_completed['6'] else ''}")
        print(f"7. Release Vested Tokens {'✔' if actions_completed['7'] else ''}")
        print(f"77. Release Vested Tokens by Coin {'✔' if actions_completed['77'] else ''}")
        print(f"8. Collect Not Vested Coins {'✔' if actions_completed['8'] else ''}")
        print(f"9. Execute a function {'✔' if actions_completed['9'] else ''}")
        print("10. Exit")
        print("11. Set Deployed Package ID")
        print("12. Set Coin ID")
        print("13. Set Strategy ID")
        print("14. Set Vester ID")

        choice = input("Enter your choice (0-14): ")

        if choice == "0":
            deploy()
            deployed_package = input("Enter the deployed package ID: ")
            actions_completed["0"] = True
        elif choice == "1":
            mint_examplecoin()
            coin_id = input("Enter the minted ID: ")
            actions_completed["1"] = True
        elif choice == "2":
            create_strategy_not_for_coin()
            strategy_id = input("Enter the created Strategy ID: ")
            actions_completed["2"] = True
        elif choice == "3":
            create_strategy_for_coin()
            strategy_id = input("Enter the created Strategy ID: ")
            actions_completed["3"] = True
        elif choice == "4":
            create_vester(strategy_id)
            vester_id = input("Enter the created vester ID: ")
            actions_completed["4"] = True
        elif choice == "5":
            transfer_object(vester_id, coin_id)
            actions_completed["5"] = True
        elif choice == "6":
            initialize_vester(vester_id, coin_id)
            actions_completed["6"] = True
        elif choice == "7":
            release_vested_tokens(vester_id)
            actions_completed["7"] = True
        elif choice == "77":
            release_vested_tokens_by_coin(vester_id, "")
            actions_completed["77"] = True
        elif choice == "8":
            collect_not_vested_coins(vester_id)
            actions_completed["8"] = True
        elif choice == "9":
            call_other_function()
            actions_completed["9"] = True
        elif choice == "10":
            print("Exiting...")
            break
        elif choice == "11":
            deployed_package = input("Enter the deployed package ID: ")
        elif choice == "12":
            coin_id = input("Enter the minted Coin ID: ")
        elif choice == "13":
            strategy_id = input("Enter the Strategy ID: ")
        elif choice == "14":
            vester_id = input("Enter the Vester ID: ")
        else:
            print("Invalid choice. Please try again.")


if __name__ == "__main__":
    main()


"""
iota client ptb \
    --move-call iota::tx_context::sender \
	--assign sender \
    --move-call 0xa4743a6554c75115db0fd36960f7c39012288daeecc6a624a0600fa65a890c0e::vesting::create_strategy_not_for_coin  2u8 10 \
    --assign strategy \
    --move-call 0xa4743a6554c75115db0fd36960f7c39012288daeecc6a624a0600fa65a890c0e::vesting::create_vester_and_share  \
    "<0x9a80f8d5de67c65b0dbe4e38cfff9624f35ee9705a2def73fd0d6a7e156ea161::ExampleCoin::EXAMPLECOIN, u64>"  \
    strategy 1726743803978 "some(1000000)" none none \
    --gas-budget 50000000

iota client ptb \
    --move-call iota::tx_context::sender \
	--assign sender \
    --move-call 0xa4743a6554c75115db0fd36960f7c39012288daeecc6a624a0600fa65a890c0e::vesting::create_and_initialize_vester  2 10 \
    "<0x9a80f8d5de67c65b0dbe4e38cfff9624f35ee9705a2def73fd0d6a7e156ea161::ExampleCoin::EXAMPLECOIN, u64>"  \
    strategy 1726743803978 1000000 none none \
    --gas-budget 50000000

    iota client ptb \
    --move-call iota::tx_context::sender \
	--assign sender \
    --move-call 0xa4743a6554c75115db0fd36960f7c39012288daeecc6a624a0600fa65a890c0e::vesting::create_vester_and_share  \
    "<0x9a80f8d5de67c65b0dbe4e38cfff9624f35ee9705a2def73fd0d6a7e156ea161::ExampleCoin::EXAMPLECOIN, u64>"  \
    @0x946e560d91f4419878d9dc5ab63adb144970f72c981d60d11b561f831d9fbb04 1726743803978 1000000 none none \
    --gas-budget 50000000


    iota client ptb \
    --move-call 0xa4743a6554c75115db0fd36960f7c39012288daeecc6a624a0600fa65a890c0e::vesting::create_vester_and_share  \
    "<0x9a80f8d5de67c65b0dbe4e38cfff9624f35ee9705a2def73fd0d6a7e156ea161::ExampleCoin::EXAMPLECOIN, u64>"  \
    @0x946e560d91f4419878d9dc5ab63adb144970f72c981d60d11b561f831d9fbb04 1726743803978 1000000 none none \
    --gas-budget 50000000

    iota client call \
    --package 0xa4743a6554c75115db0fd36960f7c39012288daeecc6a624a0600fa65a890c0e \
    --module vesting --function create_vester_and_share \
    --type-args 0x9a80f8d5de67c65b0dbe4e38cfff9624f35ee9705a2def73fd0d6a7e156ea161::ExampleCoin::EXAMPLECOIN u64 \
    --args 0x946e560d91f4419878d9dc5ab63adb144970f72c981d60d11b561f831d9fbb04 1726743803978 1000000 none none \
    --gas-budget 50000000

    iota client ptb --move-call 0xa4743a6554c75115db0fd36960f7c39012288daeecc6a624a0600fa65a890c0e::vesting::init_strategy_not_for_coin     2 10 --gas-budget 5000000
    iota client call --package  0xa4743a6554c75115db0fd36960f7c39012288daeecc6a624a0600fa65a890c0e --module vesting --function init_strategy_not_for_coin    --args 2 10 --gas-budget 5000000

"""