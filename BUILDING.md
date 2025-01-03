# Building sniffnet

### Step 1: Clone the repository and CD into repo root

```sh
git clone GyulyVGC/sniffnet
cd sniffnet
```

### Step 2: Update cargo packages

```sh
cargo update
```

### Step 3: Install `lib`

```sh
apt-get install lib
```

### Step 4: Build and Compress Binary

```sh
cargo build --release
cp target/release/sniffnet .
upx compress --brute sniffnet ( from 36MB to 12MB )
```

### Step 5: Run sniffnet

```sh
./sniffnet
```
