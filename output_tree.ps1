try {
# call output_tree_bteq.ps1 first
    Import-Csv -Path data_tmp.csv -Header DBName,OtherColumns |
        Select-Object -ExpandProperty DBName -Unique |
        Set-Content -Path data/data.csv -Encoding UTF8
    # Read unique database names into a hash table
    $dbs = @{}
    Get-Content -Path data/data.csv | ForEach-Object {
        $db = $_.Trim()
        if (-not [string]::IsNullOrWhiteSpace($db)) {
            $dbs[$db] = $true
        }
    }
$inputFile = "C:\your\path\to\data_tmp.csv"
$outputDir = "C:\your\path\to\data"
if (-not (Test-Path -Path $outputDir)) {
    New-Item -Path $outputDir -ItemType Directory | Out-Null
}


Write-Host "Importing and grouping data... (This may take a moment for a large file)"
$groupedData = Import-Csv -Path $inputFile -Delimiter ',' -Header DBName, TBName, ColumnName | Group-Object -Property DBName
Write-Host "Writing split files..."
foreach ($group in $groupedData) {
    $currentDb = $group.Name
    $outputPath = Join-Path -Path $outputDir -ChildPath "data_$($currentDb).csv"
    Write-Host "  -> Writing $($group.Count) rows to $outputPath"
    $group.Group | Export-Csv -LiteralPath $outputPath -NoTypeInformation
}
Write-Host "Processing complete."
}
catch {
    Write-Error "An error occurred: $($_.Exception.Message)"
    exit 1
}
finally {
    # Remove temporary file
    if (Test-Path -Path data_tmp.csv) {
        Remove-Item -Path data_tmp.csv -Force
    }
}
exit 0
