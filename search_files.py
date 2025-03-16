import os
import csv

def search_csv_files(directory, postal_code):
    """
    Search for a postal code in all CSV files within a specified directory using the csv module.

    :param directory: The directory containing CSV files.
    :param postal_code: The postal code to search for.
    :return: A list of rows (as dictionaries) where the postal code is found.
    """
    print(f"Searching for postal code: {postal_code} in directory: {directory}")
    results = []
    for filename in os.listdir(directory):
        print(f"Checking file: {filename}")
        if filename.endswith('.csv'):
            file_path = os.path.join(directory, filename)
            print(f"Loading file into memory: {file_path}")
            with open(file_path, mode='r', newline='', encoding='utf-8') as csvfile:
                reader = csv.DictReader(csvfile)
                for row in reader:
                    if row['postal_code'] == postal_code:
                        print(f"Match found in file: {filename}")
                        results.append(row)
    print(f"Total matches found: {len(results)}")
    return results

# Example usage
directory_path = './data/'
postal_code_to_search = '6369CW'
print(f"Directory path set to: {directory_path}")
print(f"Postal code to search: {postal_code_to_search}")
matching_rows = search_csv_files(directory_path, postal_code_to_search)

print("Matching rows:")
for row in matching_rows:
    print(row)
