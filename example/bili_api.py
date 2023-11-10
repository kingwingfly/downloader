import asyncio
import httpx


async def main():
    async with httpx.AsyncClient() as client:
        api = "https://api.bilibili.com/x/player/wbi/playurl"
        resp = await client.get(
            api,
            params={
                "bvid": "BV1EC4y1V7ho",
                "cid": 1303248673,
                "qn": 127,
                "fourk": 1,
                "fnval": 16 | 1024,
                "fnver": 0
            },
            headers={
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko)",
                "Cookie": ""
            },
        )
        print(resp.json())


if __name__ == "__main__":
    asyncio.run(main())
