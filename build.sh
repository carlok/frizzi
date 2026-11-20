#!/bin/sh
set -eu
cd "$(dirname "$0")"
python3 deploy/setup-invitation-pdf.py --bundle runtime/frizzi
printf 'Frizzi PDF runtime prepared for the Rust build.\n'
