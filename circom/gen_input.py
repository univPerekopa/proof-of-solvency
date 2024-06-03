import json
import sys

def main():
    try:
        # Read the number of elements from stdin
        n = int(input("Enter the number of elements: "))

        # Generate the list of elements as strings
        leafs = [str(i + 1) for i in range(n)]

        # Create the JSON object
        json_data = {"leafs": leafs}

        # Write the JSON object to a file
        with open('input.json', 'w') as json_file:
            json.dump(json_data, json_file, indent=4)

        print(f"JSON data has been written to input.json with {n} elements.")

    except ValueError:
        print("Please enter a valid integer.")
    except Exception as e:
        print(f"An error occurred: {e}")

if __name__ == "__main__":
    main()
