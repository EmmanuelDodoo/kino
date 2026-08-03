$Version = "1.26.6"

Invoke-WebRequest `
  -Uri "https://gstreamer.freedesktop.org/data/pkg/windows/$Version/msvc/gstreamer-1.0-msvc-x86_64-$Version.msi" `
  -OutFile "gstreamer-runtime.msi"

Invoke-WebRequest `
  -Uri "https://gstreamer.freedesktop.org/data/pkg/windows/$Version/msvc/gstreamer-1.0-devel-msvc-x86_64-$Version.msi" `
  -OutFile "gstreamer-devel.msi"

Start-Process msiexec.exe `
  -ArgumentList "/i gstreamer-runtime.msi /qn INSTALLLEVEL=1000" `
  -Wait

Start-Process msiexec.exe `
  -ArgumentList "/i gstreamer-devel.msi /qn INSTALLLEVEL=1000" `
  -Wait


"PKG_CONFIG_PATH=C:\Program Files\gstreamer\1.0\msvc_x86_64\lib\pkgconfig" >> $env:GITHUB_ENV
"PATH=C:\Program Files\gstreamer\1.0\msvc_x86_64\bin;$env:PATH" >> $env:GITHUB_ENV
