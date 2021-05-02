// WeAthFolD 2021.5.2
// https://leetcode.com/problems/lru-cache/
#include <list>
#include <unordered_map>

using namespace std;

struct LinkNode {
    LinkNode* prev;
    LinkNode* next;
    int key;

    LinkNode() : prev(nullptr), next(nullptr), key(0) {}
};

struct HashEntry {
    int val;
    LinkNode* node;
};

class LRUCache {
public:
    LRUCache(int capacity) {
        this->_cap = capacity;
        this->_head = new LinkNode();
        this->_tail = new LinkNode();
        this->_head->next = this->_tail;
        this->_tail->prev = this->_head;
    }
    
    int get(int key) {
        auto it = this->_lookup.find(key);
        if (it != this->_lookup.end()) {
            // Move this item to head of the list 
            auto node_ptr = it->second.node;

            node_ptr->prev->next = node_ptr->next;
            node_ptr->next->prev = node_ptr->prev;
            node_ptr->next = this->_head->next;
            this->_head->next->prev = node_ptr;
            node_ptr->prev = this->_head;
            this->_head->next = node_ptr;
            
            return it->second.val;
        } else {
            return -1;
        }
    }
    
    void put(int key, int value) {
        auto it = this->_lookup.find(key);
        if (it != this->_lookup.end()) { // exists, refresh order only
            it->second.val = value;
            this->get(key);
        } else { // not exist, push to lookup & head of queue
            if (this->_lookup.size() == this->_cap) {
                // evict last
                auto last_node_ptr = this->_tail->prev;
                this->_lookup.erase(last_node_ptr->key);

                auto new_end = last_node_ptr->prev;
                new_end->next = this->_tail;
                this->_tail->prev = new_end;
                delete last_node_ptr;
            }
            
            auto new_node = new LinkNode();
            new_node->key = key;
            new_node->next = this->_head->next;
            new_node->prev = this->_head;
            
            this->_head->next->prev = new_node;
            this->_head->next = new_node;

            HashEntry hash_entry;
            hash_entry.node = new_node;
            hash_entry.val = value;
            this->_lookup.emplace(key, hash_entry);
        }
    }
    
private:
    size_t _cap;
    unordered_map<int, HashEntry> _lookup;
    LinkNode* _head;
    LinkNode* _tail;
};