!macro NSIS_HOOK_PREINSTALL
  ; 检查并强制结束正在运行的旧版本进程
  ; 安装前先结束老进程，避免文件被占用无法覆盖
  nsExec::Exec '"taskkill" /im "evocode-tauri.exe" /f'
  Pop $0
  Sleep 500
  
  ; 二次确认
  nsExec::Exec '"taskkill" /im "evocode-tauri.exe" /f'
  Pop $0
  Sleep 300
!macroend
