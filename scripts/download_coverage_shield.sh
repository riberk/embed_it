#! /bin/bash
get_color() (
    local input=$1

    if ! [[ $input =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
        echo "Error: Input is not a number"
        return 1
    fi

    if (( $(echo "$input > 100" | bc -l) )); then
        echo "Error: Number is greater than 100"
        return 1
    fi

    if (( $(echo "$input < 50" | bc -l) )); then
        echo "e05d44"
    elif (( $(echo "$input < 60" | bc -l) )); then
        echo "fe7d37"
    elif (( $(echo "$input < 75" | bc -l) )); then
        echo "dfb317"
    elif (( $(echo "$input < 90" | bc -l) )); then
        echo "a4a61d"
    elif (( $(echo "$input < 95" | bc -l) )); then
        echo "97ca00"
    else
        echo "40c010"
    fi
    
)

input="${1}"
color=$(get_color "$input")

url="https://img.shields.io/badge/Coverage-${input}%25-${color}"

curl $url
