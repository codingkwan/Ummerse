$env:PATH = "C:\Users\ghx\.cargo\bin;$env:PATH"
Set-Location "c:\Users\ghx\coding\ummerse"
& "C:\Users\ghx\.cargo\bin\cargo.exe" check -p ummerse-math -p ummerse-core -p ummerse-asset -p ummerse-scene 2>&1 | Select-Object -First 200
