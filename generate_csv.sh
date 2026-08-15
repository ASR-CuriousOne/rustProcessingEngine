#!/bin/bash

INPUT_FILE="testData/customers-2000000.csv"
OUTPUT_FILE="testData/customers-massive.csv"
TARGET_SIZE_MB=4096 # Target size in megabytes (e.g., 1024 MB = 1 GB)

# 1. Extract the header and write it to the output file
head -n 1 "$INPUT_FILE" > "$OUTPUT_FILE"

# 2. Extract the data (everything except the header) to a temp file
tail -n +2 "$INPUT_FILE" > temp_data.csv

# 3. Calculate target bytes
TARGET_BYTES=$((TARGET_SIZE_MB * 1024 * 1024))
CURRENT_BYTES=$(stat -c%s "$OUTPUT_FILE")

echo "Generating file up to $TARGET_SIZE_MB MB..."

# 4. Append data in a loop until the target size is met
while [ "$CURRENT_BYTES" -lt "$TARGET_BYTES" ]; do
    cat temp_data.csv >> "$OUTPUT_FILE"
    CURRENT_BYTES=$(stat -c%s "$OUTPUT_FILE")
    
    Optional: Print progress every loop iteration
    echo -ne "Current size: $((CURRENT_BYTES / 1024 / 1024)) MB\r"
done

# 5. Clean up
rm temp_data.csv

echo -e "\nDone! Output saved to $OUTPUT_FILE"
