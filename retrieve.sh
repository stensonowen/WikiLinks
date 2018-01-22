#!/bin/sh
# simplewiki: takes ~25 seconds, unzips to ~2.1M

date=`date +%Y%0m01`
lang='simple'

base=https://dumps.wikimedia.org/$lang'wiki'/$date/
#                                           Compressed  Uncompressed
redr=$lang'wiki-'$date'-redirect.sql.gz'   # 107.6 MB
lnks=$lang'wiki-'$date'-pagelinks.sql.gz'   # 4.9 GB
page=$lang'wiki-'$date'-page.sql.gz'        # 1.4 GB

wget $base$redr 2> /dev/null && gunzip $redr &
wget $base$lnks 2> /dev/null && gunzip $lnks &
wget $base$page 2> /dev/null && gunzip $page &

wait
