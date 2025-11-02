# Web Panel Development Rules

- You MUST write source code in English and write i18n code in `server/web-panel/src/locales/en.json` and `server/web-panel/src/locales/zh.json`
- web-panel use grpc-web to connect to server, service files are in `service` folder, the typescript connection object is defined in `server/web-panel/src/stores/grpc.ts`
