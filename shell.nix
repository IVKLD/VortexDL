(import
  (fetchTarball {
    url = "https://github.com/edolstra/flake-compat/archive/master.tar.gz";
    sha256 = "1m96p9vsq27m8sh8fayizsc663673j9fhyv8n43y9q5b4idb639s";
  })
  { src = ./.; }).shellNix
