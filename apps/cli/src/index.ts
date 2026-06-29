const args = process.argv.slice(2);

if (args.length === 0) {
  console.log("aio: command surface bootstrap complete (install/remove/update/search planned).");
  process.exit(0);
}

console.log(`aio: received command '${args.join(" ")}'`);
