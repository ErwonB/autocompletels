#!/bin/bash

tbuild -f export_db.tpt || exit 1

cut -d',' -f1 data_tmp.csv | sort -u > data/data.csv

declare -A dbs
while IFS= read -r db; do
  dbs["$db"]=1
done < data/data.csv

# Use awk to filter data_all.csv based on the associative array
awk -F, -v dbs="$(printf "%s\n" "${!dbs[@]}")" '
BEGIN {
  split(dbs, db_array, "\n")
  for (i in db_array) {
    db_map[db_array[i]] = 1
  }
}
{
  if ($1 in db_map) {
    print > "data/data_" $1 ".csv"
  }
}' data_tmp.csv

[[ -e "data_tmp.csv" ]] && rm data_tmp.csv

exit 0

