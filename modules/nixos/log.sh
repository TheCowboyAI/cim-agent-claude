echo 'graph TD;'

git log --all --pretty=format:'%h %p' | while read line; do
  commit_hash=$(echo $line | cut -d' ' -f1)
  parent_hashes=$(echo $line | cut -d' ' -f2-)

  if [[ -n "$parent_hashes" ]]; then
    for parent_hash in $parent_hashes; do
      echo "    ${parent_hash} --> ${commit_hash};"
    done
  fi
done
