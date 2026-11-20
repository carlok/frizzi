import EmbeddedPostgres from 'embedded-postgres';
import { spawn } from 'node:child_process';
import { join } from 'node:path';
import { mkdir, access } from 'node:fs/promises';
import { toolsDir, mailpitPath, setupTools } from './setup-tools.mjs';
await setupTools();
const databaseDir=join(toolsDir,'postgres-16');
await mkdir(databaseDir,{recursive:true});
const database=new EmbeddedPostgres({databaseDir,user:'frizzi',password:'frizzi',port:55432,persistent:true,onLog:()=>{},onError:line=>console.error(line)});
try{await access(join(databaseDir,'PG_VERSION'));}catch{await database.initialise();}
await database.start();
const client=database.getPgClient('postgres');await client.connect();
if(!(await client.query("SELECT 1 FROM pg_database WHERE datname='frizzi'")).rowCount)await database.createDatabase('frizzi');
await client.end();
const smtpPort = process.env.FRIZZI_SMTP_PORT || '11025';
const mailpitPort = process.env.FRIZZI_MAILPIT_PORT || '18025';
const mailpit=spawn(mailpitPath,[],{stdio:'inherit',env:{...process.env,MP_SMTP_BIND_ADDR:`127.0.0.1:${smtpPort}`,MP_UI_BIND_ADDR:`127.0.0.1:${mailpitPort}`}});
console.log(`PostgreSQL 16: localhost:55432 · Mailpit: http://localhost:${mailpitPort} · SMTP: localhost:${smtpPort}`);
console.log(`Set DATABASE_URL=postgres://frizzi:frizzi@localhost:55432/frizzi and SMTP_PORT=${smtpPort} in .env.`);
let stopping=false;
async function stop(){if(stopping)return;stopping=true;mailpit.kill();await database.stop();process.exit();}
process.on('SIGINT',stop);process.on('SIGTERM',stop);mailpit.on('exit',stop);
