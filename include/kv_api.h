#pragma once

#include "rust/cxx.h"
#include <unordered_map>
#include <string>

class KVInterface {
public:
    virtual ~KVInterface() = default;

    // Return a view into the stored rust::Vec
    virtual rust::Slice<const uint8_t> get(rust::Str key) {
        auto it = store.find(std::string(key.data(), key.size()));
        if (it == store.end()) {
            return rust::Slice<const uint8_t>();
        }
        // Return a slice view into the rust::Vec
        return rust::Slice<const uint8_t>(it->second.data(), it->second.size());
    }

    // Takes ownership of rust::Vec<uint8_t> and stores it directly
    virtual void set(rust::Str key, rust::Vec<uint8_t> data) {
        store[std::string(key.data(), key.size())] = std::move(data);
    }

    virtual bool delete_key(rust::Str key) {
        return store.erase(std::string(key.data(), key.size())) > 0;
    }

protected:
    std::unordered_map<std::string, rust::Vec<uint8_t>> store; // Store rust::Vec directly!
};
