# aitalked
![unsafe 100%](https://github.com/rochacbruno/rust_memes/blob/master/img/unsafe_badge.jpg)

W.I.P. GynoidTalk / VOICEROID2 Low-Level Rust Binding Library based on @KOBA789 works.

## Related Projects
- https://github.com/Nkyoku/pyvcroid2 - High-Level Python Binding of aitalked.dll
- https://github.com/Nkyoku/voiceroid_daemon - C# WebServer with Low-Level C# Binding of aitalked.dll
- https://github.com/wallstudio/Vtil/ - Low-Level C++ Binding of aitalked.dll

## Implemented Endpoints
- _AITalkAPI_Init@4
- _AITalkAPI_LangLoad@4
- _AITalkAPI_LangClear@0
- _AITalkAPI_VoiceLoad@4
- _AITalkAPI_VoiceClear@0
- _AITalkAPI_SetParam@4
- _AITalkAPI_GetParam@8
- _AITalkAPI_TextToKana@12
- _AITalkAPI_GetKana@20
- _AITalkAPI_CloseKana@8
- _AITalkAPI_TextToSpeech@12
- _AITalkAPI_CloseSpeech@8
- _AITalkAPI_GetData@16
- _AITalkAPI_GetStatus@8
- _AITalkAPI_ReloadPhraseDic@4
- _AITalkAPI_ReloadWordDic@4
- _AITalkAPI_ReloadSymbolDic@4

## Unimplemented Endpoints
- _AITalkAPI_End@0
- _AITalkAPI_GetJeitaControl@8
- _AITalkAPI_BLoadWordDic@0
- _AITalkAPI_ModuleFlag@0
- _AITalkAPI_LicenseDate@4
- _AITalkAPI_LicenseInfo@16
- _AITalkAPI_VersionInfo@16
