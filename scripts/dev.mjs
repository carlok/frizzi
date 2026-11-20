import { spawn } from 'node:child_process';
import { homedir } from 'node:os';
import { join } from 'node:path';
const cargo=process.env.CARGO||join(homedir(),'.cargo/bin/cargo');
const children=[spawn(cargo,['run','--bin','local'],{stdio:'inherit'}),spawn(process.execPath,['node_modules/vite/bin/vite.js','dev'],{stdio:'inherit'})];
function stop(){for(const child of children)child.kill('SIGTERM');}
process.on('SIGINT',stop);process.on('SIGTERM',stop);
for(const child of children)child.on('exit',stop);
