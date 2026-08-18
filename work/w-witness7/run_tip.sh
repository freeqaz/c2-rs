#!/usr/bin/env bash
cd "$(dirname "$0")/../.." || exit 1
set -u
bash work/w-witness7/campaign.sh T1 --clean
for m in M-CS3 M-CS3B M-CS3C M-CS4 M-CS9 M-CA6 M-CA8 M-B2 M-B7; do
  bash work/w-witness7/campaign.sh "$m.tip" "$m"
done
bash work/w-witness7/campaign.sh C1b C1
bash work/w-witness7/campaign.sh M-B9.tip M-B9
echo "=== TIP CAMPAIGN DONE"
