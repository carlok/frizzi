import { mkdir, writeFile, chmod, access } from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';

export const toolsDir=join(homedir(),'.cache','frizzi');
export const mailpitPath=join(toolsDir,'mailpit');
export async function setupTools(){
 try{await access(mailpitPath);return;}catch{}
 if(process.platform!=='linux'||process.arch!=='x64')throw new Error('Use Docker Compose or install Mailpit manually on this platform.');
 await mkdir(toolsDir,{recursive:true});
 const version='v1.34.0';
 const base=`https://github.com/axllent/mailpit/releases/download/${version}`;
 const archive='mailpit-linux-amd64.tar.gz';
 const response=await fetch(`${base}/${archive}`);if(!response.ok)throw new Error('Mailpit download failed');
 const blob=Buffer.from(await response.arrayBuffer());
 const checksumResponse=await fetch(`${base}/checksums.txt`);if(!checksumResponse.ok)throw new Error('Mailpit checksums unavailable');
 const sums=await checksumResponse.text();
 const expected=sums.split('\n').find(line=>line.endsWith(archive))?.split(/\s+/)[0];
 if(!expected||createHash('sha256').update(blob).digest('hex')!==expected)throw new Error('Mailpit checksum mismatch');
 const tarPath=join(toolsDir,archive);await writeFile(tarPath,blob);
 execFileSync('tar',['-xzf',tarPath,'-C',toolsDir,'mailpit']);await chmod(mailpitPath,0o755);
 console.log(`Mailpit ${version} installed.`);
}
if(process.argv[1]===new URL(import.meta.url).pathname)await setupTools();
