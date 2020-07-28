# Notes

#### To Do
* ~~Finish the convertion of Flow to DialogFlow~~
* ~~Finish the convertion of Dialog to Transaction (check states from the RFC)~~
* ~~Finsih the generation of the Response from the incoming Request~~
* ~~Finish the processor for the registration~~
* ~~Fix sip trace to log the whole message + the struct somehow~~
* ~~Look into traits~~
* Probably we need to save all contact params in a hashmap in the store


Much later:
* ~~convert the state machine to use traits and generics~~
* fix upsert
* remove asyncs from store since await is not used
* probably we need to revert the from/into traits to be declared in the store


#### Viska notes
I need to follow through RFC on the necessary processes regarding:
* receiving a request
* receiving a response
* sending a request
* sending a response

Also big note on stateless proxies section, where it basically says that stateless
proxies are mostly used to handle unauthorized requests. Hence an architecture should
make it easy to specify, based on request method/headers/uri/etc what type of UAS
to run.

```
REGISTER sip:192.168.1.223 SIP/2.0
Via: SIP/2.0/UDP 192.168.1.223:5066;rport;branch=z9hG4bKPjad27b61a-897c-47e0-abaf-d8503343d398
Max-Forwards: 70
From: "vasilakisfil" <sip:vasilakisfil@192.168.1.223>;tag=80535569-c5ea-40cd-9742-8a9ab178eede
To: "vasilakisfil" <sip:vasilakisfil@192.168.1.223>
Contact: <sip:18902673@192.168.1.223:5066>;+sip.instance="<urn:uuid:1e020c2b-46f6-4867-9d11-65547b8967fa>"
Call-ID: d288a75c-6280-4330-886d-ff0df221696b
CSeq: 1 REGISTER
Expires: 600
Supported: gruu
User-Agent: Blink 3.2.1 (Linux)
Content-Length:  0
```
