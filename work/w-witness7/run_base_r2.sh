#!/usr/bin/env bash
cd "$(dirname "$0")/../.." || exit 1
set -u
bash work/w-witness7/campaign.sh N0.r2 --clean
bash work/w-witness7/campaign.sh C1a.r2 C1
for m in M-CS3 M-CS3B M-CS3C M-CS4 M-CS9 M-CA6 M-CA8 M-B2 M-B7 M-B9; do
  bash work/w-witness7/campaign.sh "$m.base.r2" "$m"
done
bash work/w-witness7/corpus.sh N1r2
echo "=== BASE r2 DONE"
