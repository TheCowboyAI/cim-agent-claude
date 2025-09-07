#! /usr/bin/env nix-shell
#! nix-shell -i bash -p cacert curl jq unzip
# shellcheck shell=bash
set -eu -o pipefail

OUTFILE="vsextensions.nix"

# Helper to just fail with a message and non-zero exit code.
function fail() {
    echo "$1" >&2
    exit 1
}

# Helper to clean up after ourselves if we're killed by SIGINT.
function clean_up() {
    TDIR="${TMPDIR:-/tmp}"
    echo "Script killed, cleaning up tmpdirs: $TDIR/vscode_exts_*" >&2
    rm -Rf "$TDIR/vscode_exts_*"
}

function get_vsixpkg() {
    N="$1.$2"

    # Create a tempdir for the extension download.
    EXTTMP=$(mktemp -d -t vscode_exts_XXXXXXXX)

    URL="https://$1.gallery.vsassets.io/_apis/public/gallery/publisher/$1/extension/$2/latest/assetbyname/Microsoft.VisualStudio.Services.VSIXPackage"

    # Quietly but delicately curl down the file, blowing up at the first sign of trouble.
    curl --silent --show-error --retry 3 --fail -X GET -o "$EXTTMP/$N.zip" "$URL"
    
    # Unpack the file we need to stdout then pull out the version
    if ! VER=$(jq -r '.version' <(unzip -qc "$EXTTMP/$N.zip" "extension/package.json")); then
        echo "Error processing $N.zip" >&2
        return 1
    fi

    # Calculate the SHA
    SHA=$(nix-hash --flat --base32 --type sha256 "$EXTTMP/$N.zip")

    # Clean up.
    rm -Rf "$EXTTMP"

    # Return the string instead of directly appending to the file
    echo "  { name = \"$2\"; publisher = \"$1\"; version = \"$VER\"; sha256 = \"$SHA\"; }"
}

# set our ext inputs
EXTS=$(<vscode-extensions.txt)

# Try to be a good citizen and clean up after ourselves if we're killed.
trap clean_up SIGINT

# Write the header to the outfile
echo -e "{\nextensions = [\n" > "$OUTFILE"

# Process each extension
for i in $EXTS; do
    OWNER=$(echo "$i" | cut -d. -f1)
    EXT=$(echo "$i" | cut -d. -f2)

    EXTENSION_DATA=$(get_vsixpkg "$OWNER" "$EXT")
    if [ $? -eq 0 ]; then
        echo "$EXTENSION_DATA" >> "$OUTFILE"
        # printf "$EXT\n"
    else
        echo "Failed to process extension $i" >&2
    fi
done

# Write the footer to the outfile
echo -e "];\n}\n" >> "$OUTFILE"
#printf "done.\nReady for rebuild"
