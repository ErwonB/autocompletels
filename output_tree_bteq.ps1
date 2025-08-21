<#
.SYNOPSIS
    A PowerShell script to export data from Teradata using BTEQ.
    Generated with AI

.DESCRIPTION
    This script connects to a Teradata database using the credentials provided,
    executes a SELECT query, and saves the results to a CSV file.
    It is a conversion of an original BASH script.
#>

# --- Configuration Variables ---
$user = "USERNAME"
# The password uses the Teradata Wallet reference. PowerShell handles the '$' character,
# so we enclose it in single quotes to treat it as a literal string.
$pwd = '$tdwallet(USERNAME)'
$tdp = "TDPID"
# Define file paths
$basePath = "C:\your\path\to\data"
$outfile = Join-Path -Path $basePath -ChildPath "data_tmp.csv"
$errfile = Join-Path -Path $basePath -ChildPath "data_tmp.err"

# --- Script Execution ---
# 1. Clean up the output file if it already exists (equivalent to [[ -f ... ]] && rm)
if (Test-Path -Path $outfile -PathType Leaf) {
    Write-Host "Removing existing output file: $outfile"
    Remove-Item -Path $outfile -Force
}

# 2. Define the BTEQ commands using a PowerShell "here-string" (equivalent to BASH's <<EOF)
#    The here-string starts with @" and ends with "@ on a new line.
$bteqScript = @"
.logmech LDAP
.logon $tdp/$user,$pwd
.if ErrorLevel <> 0 then .goto LOGON_FAIL;
SET QUERY_BAND='UserName=bourren;Origine=BTEQ;' FOR SESSION;
.set titledashes off;
.set width 3000;
.set maxerror 4;
.export report file="$outfile";
lock row for access
SELECT
    trim(c.databasename) || ',' || trim(c.tablename) || ',' || trim(c.columnname)
from dbc.tablesv t
inner join dbc.columnsv c
    on t.databasename = c.databasename
    and t.tablename = c.tablename
where t.tablekind in ('T', 'V')
order by c.databasename, c.tablename, c.columnid;
.export reset;
.exit 0;
.Label LOGON_FAIL;
.exit 2;
"@

# 3. Execute BTEQ by piping the script string into the bteq executable.
#    All output (standard and error streams) is redirected to the error file.
Write-Host "Starting BTEQ export..."
$bteqScript | bteq > $errfile 2>&1

# 4. Check the exit code of the last-run native command ($LASTEXITCODE)
if ($LASTEXITCODE -eq 0) {
    # Success
    Write-Host "Export completed successfully."
    # Clean up the (likely empty) error file
    if (Test-Path -Path $errfile) {
        Remove-Item -Path $errfile -Force
    }
    # Set exit code for the PowerShell script itself
    exit 0
} else {
    # Failure
    # Write-Error is the idiomatic PowerShell way to show terminating errors
    Write-Error "Error exporting data. BTEQ exited with code: $LASTEXITCODE"
    # Display the contents of the BTEQ error log
    if (Test-Path -Path $errfile) {
        Write-Error "--- BTEQ Error Log Content ---"
        Get-Content -Path $errfile | ForEach-Object { Write-Error $_ }
        # Clean up the error file
        Remove-Item -Path $errfile -Force
    }
    # Clean up the potentially incomplete output file
    if (Test-Path -Path $outfile) {
        Remove-Item -Path $outfile -Force
    }
    # Exit the PowerShell script with the error code from BTEQ
    exit $LASTEXITCODE
}
