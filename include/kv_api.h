#pragma once

#include "rust/cxx.h"
#include <vector>
#include <unordered_map>
#include <string>

class KVInterface {
public:
    virtual ~KVInterface() = default;

    // Return a rust::Slice (cxx maps this to Rust &[u8])
    virtual rust::Slice<const uint8_t> get(rust::Str key) {
        auto it = store.find(std::string(key.data(), key.size()));
        if (it == store.end()) {
            return rust::Slice<const uint8_t>(); // empty slice => None
        }
        return rust::Slice<const uint8_t>(it->second.data(), it->second.size());
    }

    // Accept a rust::Slice for the value
    virtual void set(rust::Str key, rust::Slice<const uint8_t> data) {
        store[std::string(key.data(), key.size())] =
            std::vector<uint8_t>(data.data(), data.data() + data.size());
    }

    virtual bool delete_key(rust::Str key) {
        return store.erase(std::string(key.data(), key.size())) > 0;
    }

private:
    std::unordered_map<std::string, std::vector<uint8_t>> store;
};
